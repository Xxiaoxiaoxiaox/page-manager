const invoke = (window.__TAURI__?.core?.invoke) || null;
let apps = [], currentIndex = 0, proxyLocal = "", proxy7897 = "";
let sidebarWidth = parseInt(localStorage.getItem("sidebarWidth")) || 220;
let dragIdx = -1;
const proxyLabels = {"local":"本地代理", "7897":"7897 代理"};

async function loadProxies() {
  if (!invoke) return;
  var p = await invoke("get_proxies");
  proxyLocal = p[0]; proxy7897 = p[1];
  updateProxyBadge();
}
function getCurrentProxyLabel() {
  var app = apps[currentIndex];
  if (!app || !app.proxy || app.proxy === "direct") return "直连";
  return proxyLabels[app.proxy] || app.proxy;
}
function updateProxyBadge() {
  var btn = document.getElementById("proxyBtn");
  if (btn) btn.textContent = "🌐 " + getCurrentProxyLabel();
}
async function loadApps() { if (!invoke) return; apps = await invoke("get_apps"); await loadProxies(); renderAppList(); var sb = document.getElementById("sidebar"); if (sb) sb.style.width = sidebarWidth + "px"; if (apps.length > 0) selectApp(0); }
function renderAppList() { var list = document.getElementById("appList"); if (!list) return; var h = ""; for (var i = 0; i < apps.length; i++) { var a = apps[i]; var c = "app-item"; if (i === currentIndex) c += " active"; if (a.exe_path) c += " has-exe"; h += '<div class="' + c + '" data-index="' + i + '">'; h += '<div class="icon icon-' + a.name.toLowerCase() + '">' + a.name.charAt(0) + '</div>'; h += '<div class="info"><div class="name">' + a.name + '</div>'; h += '<div class="url">' + new URL(a.url).hostname + '</div></div>'; h += '<div class="badge"></div></div>'; } list.innerHTML = h; var items = list.querySelectorAll(".app-item"); for (var i = 0; i < items.length; i++) { (function(idx) { items[idx].addEventListener("click", function() { selectApp(idx); }); items[idx].addEventListener("dblclick", function() { if (apps[idx].exe_path && invoke) invoke("launch_app", { exePath: apps[idx].exe_path }); else selectApp(idx); }); })(i); } }
function selectApp(index) { currentIndex = index; var app = apps[index]; if (!app) return; var items = document.querySelectorAll(".app-item"); for (var i = 0; i < items.length; i++) items[i].classList.remove("active"); var ae = document.querySelector('.app-item[data-index="' + index + '"]'); if (ae) ae.classList.add("active"); var te = document.querySelector(".toolbar .title"); if (te) te.textContent = new URL(app.url).hostname; updateProxyBadge(); if (invoke) invoke("navigate_to", { url: app.url, sidebarWidth: sidebarWidth }).catch(function(){}); }
document.getElementById("themeToggle").addEventListener("click", function() {
  var body = document.body;
  var btn = document.getElementById("themeToggle");
  if (body.getAttribute("data-theme") === "dark") {
    body.setAttribute("data-theme", "light");
    btn.textContent = "☀️";
  } else {
    body.setAttribute("data-theme", "dark");
    btn.textContent = "🌙";
  }
});
document.getElementById("addBtn").addEventListener("click", function() { showAppForm(null); });
document.getElementById("manageBtn").addEventListener("click", showManagePanel);
document.getElementById("proxyBtn").addEventListener("click", showProxyPanel);
document.getElementById("modalClose").addEventListener("click", closeModal);
document.getElementById("modalOverlay").addEventListener("click", function(e) { if (e.target === e.currentTarget) closeModal(); });

async function hideContent() { if (invoke) await invoke("toggle_content_webview", { visible: false }); }
async function showContent() { if (invoke) await invoke("toggle_content_webview", { visible: true }); }
async function closeModal() { await showContent(); document.getElementById("modalOverlay").classList.remove("open"); }

function showAppForm(index) {
  hideContent();
  var app = (index !== null) ? apps[index] : null;
  document.getElementById("modalTitle").textContent = app ? "编辑" : "添加应用";
  document.getElementById("modalBody").innerHTML =
    '<div class="form-group"><label>名称</label><input id="formName" value="' + (app ? app.name : "") + '"></div>' +
    '<div class="form-group"><label>网址</label><input id="formUrl" value="' + (app ? app.url : "") + '"></div>' +
    '<div class="modal-actions"><button class="btn btn-secondary" onclick="closeModal()">取消</button>' +
    '<button class="btn btn-primary" onclick="' + (app ? "saveEdit(" + index + ")" : "saveNew()") + '">保存</button></div>';
  document.getElementById("modalOverlay").classList.add("open");
}
async function saveNew() {
  var n = document.getElementById("formName").value.trim();
  var u = document.getElementById("formUrl").value.trim();
  if (!n || !u) return;
  if (invoke) await invoke("add_app", { name: n, url: u, exePath: "", proxy: "" });
  closeModal();
  loadApps();
}
async function saveEdit(i) {
  var n = document.getElementById("formName").value.trim();
  var u = document.getElementById("formUrl").value.trim();
  if (!n || !u) return;
  if (invoke) await invoke("update_app", { index: i, name: n, url: u, exePath: "", proxy: "" });
  closeModal();
  loadApps();
}
async function deleteApp(i) {
  if (!confirm("确定删除 " + apps[i].name + " 吗？")) return;
  if (invoke) await invoke("remove_app", { index: i });
  if (currentIndex === i && i > 0) currentIndex--;
  loadApps();
}

function showManagePanel() {
  hideContent();
  document.getElementById("modalTitle").textContent = "管理应用";
  var html = '<div class="app-manage-list" id="appManageList">';
  for (var i = 0; i < apps.length; i++) {
    var a = apps[i];
    var pi = a.proxy ? " · " + (proxyLabels[a.proxy] || a.proxy) : "";
    html += '<div class="app-list-item" data-idx="' + i + '">';
    html += '<div class="drag-handle" onmousedown="startMouseDrag(event,' + i + ')">⠿</div>';
    html += '<div class="info"><div class="name">' + a.name + '</div><div class="url">' + a.url + pi + '</div></div>';
    html += '<div class="actions"><button onclick="showAppForm(' + i + ')">编辑</button><button class="delete-btn" onclick="deleteApp(' + i + ')">删除</button></div></div>';
  }
  html += '</div>';
  document.getElementById("modalBody").innerHTML = html;
  document.getElementById("modalBody").innerHTML += '<div class="modal-actions"><button class="btn btn-secondary" onclick="closeModal()">关闭</button></div>';
  document.getElementById("modalOverlay").classList.add("open");
}

var mouseDragIdx = -1;

function startMouseDrag(e, idx) {
  if (e.button !== 0) return; // left click only
  mouseDragIdx = idx;
  var items = document.querySelectorAll("#appManageList .app-list-item");
  items[idx].classList.add("dragging");

  function onMove(ev) {
    ev.preventDefault();
    // Find which item we're over
    var target = ev.target.closest ? ev.target.closest(".app-list-item") : null;
    if (!target) return;
    var overIdx = parseInt(target.dataset.idx);
    if (overIdx === mouseDragIdx) return;
    // Highlight the target
    items.forEach(function(it) { it.classList.remove("drag-over"); });
    target.classList.add("drag-over");
  }

  function onUp(ev) {
    document.removeEventListener("mousemove", onMove);
    document.removeEventListener("mouseup", onUp);
    items.forEach(function(it) { it.classList.remove("dragging", "drag-over"); });
    var target = ev.target.closest ? ev.target.closest(".app-list-item") : null;
    if (!target) return;
    var toIdx = parseInt(target.dataset.idx);
    if (mouseDragIdx === -1 || mouseDragIdx === toIdx) {
      mouseDragIdx = -1;
      return;
    }
    // Reorder
    var moved = apps.splice(mouseDragIdx, 1)[0];
    apps.splice(toIdx, 0, moved);
    mouseDragIdx = -1;
    if (invoke) invoke("reorder_apps", { apps: apps }).catch(function(){});
    renderAppList();
    showManagePanel();
  }

  document.addEventListener("mousemove", onMove);
  document.addEventListener("mouseup", onUp);
}
function showProxyPanel() {
  hideContent();
  var app = apps[currentIndex];
  var pv = app ? (app.proxy || "") : "";
  var c1 = (pv === "local") ? "checked" : "";
  var c2 = (pv === "7897") ? "checked" : "";
  document.getElementById("modalTitle").textContent = "代理设置";
  document.getElementById("modalBody").innerHTML =
    '<div class="form-group"><label>本地代理地址</label><input id="proxyLocalInput" value="' + proxyLocal + '" placeholder="例: http://127.0.0.1:7890"></div>' +
    '<div class="form-group"><label>7897 代理地址</label><input id="proxy7897Input" value="' + proxy7897 + '" placeholder="http://127.0.0.1:7897"></div>' +
    '<div class="proxy-divider"></div>' +
    '<div class="form-group"><label>当前应用：' + (app ? app.name : "") + '</label>' +
    '<div class="proxy-radio-group">' +
    '<label class="proxy-radio"><input type="radio" name="proxySelect" value="local" ' + c1 + '> 本地代理</label>' +
    '<label class="proxy-radio"><input type="radio" name="proxySelect" value="7897" ' + c2 + '> 7897 代理</label>' +
    '</div></div>' +
    '<div class="modal-actions"><button class="btn btn-secondary" onclick="closeModal()">取消</button>' +
    '<button class="btn btn-primary" onclick="saveProxies()">保存</button></div>';
  document.getElementById("modalOverlay").classList.add("open");
}
async function saveProxies() {
  var l = document.getElementById("proxyLocalInput").value.trim();
  var s = document.getElementById("proxy7897Input").value.trim();
  var r = document.querySelector('input[name="proxySelect"]:checked');
  var pv = r ? r.value : "";

  // 保存代理地址
  if (invoke) { await invoke("set_proxies", { local: l, proxy7897: s }); proxyLocal = l; proxy7897 = s; }

  // 更新当前应用的代理选择
  if (invoke && apps[currentIndex]) {
    var a = apps[currentIndex];
    if (a.proxy !== pv) {
      await invoke("update_app", { index: currentIndex, name: a.name, url: a.url, exePath: a.exe_path, proxy: pv });
      apps[currentIndex].proxy = pv;
    }
  }
  updateProxyBadge();
  closeModal();
}
(function() {
  var sidebar = document.getElementById("sidebar");
  var handle = document.getElementById("resizeHandle");
  if (!sidebar || !handle) return;
  sidebar.style.width = sidebarWidth + "px";
  handle.addEventListener("mousedown", function(e) {
    e.preventDefault();
    handle.classList.add("active");
    document.body.style.cursor = "col-resize";
    document.body.style.userSelect = "none";
    function doResize(e) { sidebarWidth = Math.max(150, Math.min(500, e.clientX)); sidebar.style.width = sidebarWidth + "px"; }
    function stopResize() {
      document.removeEventListener("mousemove", doResize);
      document.removeEventListener("mouseup", stopResize);
      handle.classList.remove("active");
      document.body.style.cursor = "";
      document.body.style.userSelect = "";
      if (invoke) invoke("set_sidebar_width", { width: sidebarWidth }).catch(function(){});
      localStorage.setItem("sidebarWidth", sidebarWidth);
      if (invoke && apps[currentIndex]) invoke("navigate_to", { url: apps[currentIndex].url, sidebarWidth: sidebarWidth }).catch(function(){});
    }
    document.addEventListener("mousemove", doResize);
    document.addEventListener("mouseup", stopResize);
  });
})();

loadApps();
