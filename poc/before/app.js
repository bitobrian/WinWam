const categories = [
  ["All", "A", null], ["Bags", "B", "bags.png"], ["Collections", "C", "collections.png"],
  ["Combat", "C", "combat.png"], ["Development", "D", null], ["Dungeons", "D", "dungeon.png"],
  ["Economy", "E", "economy.png"], ["Interface", "I", "interface.png"],
  ["Quality of Life", "Q", "qualityoflife.png"], ["Quests", "Q", "questing.png"], ["Raiding", "R", null]
];

const authors = ["Lua Workshop", "Copper Byte", "Waypoint Works", "Azeroth Labs"];
const categoryNames = categories.slice(1).map(([name]) => name);
const addons = Array.from({ length: 50 }, (_, index) => {
  const number = String(index + 1).padStart(2, "0");
  const category = categoryNames[index % categoryNames.length];
  return {
    id: index + 1,
    name: `Test Addon ${number}2`,
    author: authors[index % authors.length],
    category,
    description: `Synthetic ${category.toLowerCase()} addon for local integration testing.`
  };
});

const state = { view: "browse", category: "All", query: "", sort: "name", page: 1, pageSize: 9, installed: new Set() };
const $ = selector => document.querySelector(selector);
const grid = $("#addonGrid");
const emptyState = $("#emptyState");
const dialog = $("#detailsDialog");
let toastTimer;

function renderCategories() {
  $("#categories").innerHTML = categories.map(([name, initial, image]) => `
    <button class="category ${state.category === name ? "active" : ""}" data-category="${name}" aria-pressed="${state.category === name}">
      ${image ? `<img src="../../assets/ribbon-bar/${image}" alt="">` : `<span class="initial">${initial}</span>`}
      <span>${name}</span>
    </button>`).join("");
}

function filteredAddons() {
  const query = state.query.toLowerCase();
  return addons
    .filter(addon => state.view !== "installed" || state.installed.has(addon.id))
    .filter(addon => state.category === "All" || addon.category === state.category)
    .filter(addon => `${addon.name} ${addon.author} ${addon.category}`.toLowerCase().includes(query))
    .sort((a, b) => a[state.sort].localeCompare(b[state.sort], undefined, { numeric: true }));
}

function render() {
  renderCategories();
  const filtered = filteredAddons();
  const pageCount = Math.max(1, Math.ceil(filtered.length / state.pageSize));
  state.page = Math.min(state.page, pageCount);
  const start = (state.page - 1) * state.pageSize;
  const visible = filtered.slice(start, start + state.pageSize);

  grid.innerHTML = visible.map(addon => {
    const installed = state.installed.has(addon.id);
    return `<article class="addon-card">
      <div class="card-heading"><div class="addon-avatar">TA</div><div><h2>${addon.name}</h2><p class="author">by ${addon.author}</p></div></div>
      <p class="description">${addon.description}</p>
      <div class="card-actions"><span class="tag">${addon.category}</span><button data-details="${addon.id}">Details</button><button class="install ${installed ? "installed" : ""}" data-install="${addon.id}">${installed ? "Remove" : "Install"}</button></div>
    </article>`;
  }).join("");

  emptyState.hidden = visible.length > 0;
  $("#pageStat").textContent = `Page ${state.page} of ${pageCount}`;
  $("#totalStat").textContent = `${filtered.length} addon${filtered.length === 1 ? "" : "s"}`;
  $("#installStat").textContent = `${state.installed.size} installed`;
  $("#navInstalled").textContent = `(${state.installed.size})`;
  $("#pageRange").textContent = filtered.length ? `${start + 1}–${start + visible.length} of ${filtered.length}` : "0 of 0";
  $("#previousPage").disabled = state.page === 1;
  $("#nextPage").disabled = state.page === pageCount;
}

function setView(view) {
  state.view = view;
  state.page = 1;
  document.querySelectorAll(".nav-item").forEach(button => button.classList.toggle("active", button.dataset.view === view));
  const copy = {
    browse: ["Browse Addons", "Discover and install addons from the community"],
    installed: ["Installed Addons", "Manage addons installed on this device"],
    loadouts: ["Loadouts", "Build and switch between addon collections"]
  }[view];
  $("#viewTitle").textContent = copy[0];
  $("#viewSubtitle").textContent = copy[1];
  if (view === "loadouts") showToast("Loadout editing will be explored in the next POC pass.");
  render();
}

function toggleInstall(id) {
  const addon = addons.find(item => item.id === id);
  if (state.installed.has(id)) {
    state.installed.delete(id);
    showToast(`${addon.name} removed`);
  } else {
    state.installed.add(id);
    showToast(`${addon.name} installed`);
  }
  render();
}

function openDetails(id) {
  const addon = addons.find(item => item.id === id);
  $("#dialogCategory").textContent = addon.category;
  $("#dialogTitle").textContent = addon.name;
  $("#dialogAuthor").textContent = `by ${addon.author}`;
  $("#dialogDescription").textContent = addon.description;
  const button = $("#dialogInstall");
  button.textContent = state.installed.has(id) ? "Remove addon" : "Install addon";
  button.dataset.id = id;
  dialog.showModal();
}

function showToast(message) {
  const toast = $("#toast");
  toast.textContent = message;
  toast.classList.add("show");
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => toast.classList.remove("show"), 2200);
}

$("#categories").addEventListener("click", event => {
  const button = event.target.closest("[data-category]");
  if (!button) return;
  state.category = button.dataset.category;
  state.page = 1;
  render();
});

grid.addEventListener("click", event => {
  const install = event.target.closest("[data-install]");
  const details = event.target.closest("[data-details]");
  if (install) toggleInstall(Number(install.dataset.install));
  if (details) openDetails(Number(details.dataset.details));
});

document.querySelectorAll(".nav-item").forEach(button => button.addEventListener("click", () => setView(button.dataset.view)));
$("#search").addEventListener("input", event => { state.query = event.target.value; state.page = 1; render(); });
$("#sort").addEventListener("change", event => { state.sort = event.target.value; state.page = 1; render(); });
$("#previousPage").addEventListener("click", () => { state.page -= 1; render(); scrollTo({ top: 0, behavior: "smooth" }); });
$("#nextPage").addEventListener("click", () => { state.page += 1; render(); scrollTo({ top: 0, behavior: "smooth" }); });
$("#releasePicker").addEventListener("change", event => showToast(`Switched mock catalog to ${event.target.value}`));
$(".settings").addEventListener("click", () => showToast("Settings is a placeholder in this POC."));
$(".dialog-close").addEventListener("click", () => dialog.close());
dialog.addEventListener("click", event => { if (event.target === dialog) dialog.close(); });
$("#dialogInstall").addEventListener("click", event => { toggleInstall(Number(event.target.dataset.id)); dialog.close(); });

render();
