const categories = [
  ["Bags", "bags.png"], ["Collections", "collections.png"], ["Combat", "combat.png"],
  ["Dungeons", "dungeon.png"], ["Economy", "economy.png"], ["Interface", "interface.png"],
  ["Quality of Life", "qualityoflife.png"], ["Quests", "questing.png"]
];
const authors = ["Lua Workshop", "Copper Byte", "Waypoint Works", "Azeroth Labs"];
const names = ["Bag Commander", "Collector's Journal", "Combat Rhythm", "Dungeon Compass", "Auction Lens", "Clean Interface", "Easy Questing", "Raid Signals", "Waypoint Pro", "Inventory Insight", "Mythic Notes", "Quest Tracker"];
const addons = Array.from({length:36},(_,i)=>({ id:i+1, name:i<names.length?names[i]:`Community Addon ${i+1}`, author:authors[i%4], category:categories[i%categories.length][0], image:categories[i%categories.length][1], downloads:`${12+i*3}.${i%10}k`, description:"A polished community addon designed to make every session in Azeroth smoother." }));
const state={view:"installed",category:"Bags",query:"",installed:new Set([1,2,3,4,5,6])};
const $=s=>document.querySelector(s); let toastTimer;
const skuArtwork={
  retail:{src:"../../assets/generated/midnight-logo.png",alt:"World of Warcraft Midnight"},
  forever:{src:"../../assets/generated/forever-logo.png",alt:"World of Warcraft Forever"}
};
const fileLessons={
  toc:["THE MANIFEST","MyFirstAddon.toc","The TOC tells WoW what your addon is called, which game interface it supports, and which files to load.",`## Interface: 120000\n## Title: My First Addon\n## Notes: Learning addon basics\n## Author: Your Name\n## Version: 0.1.0\n\nMyFirstAddon.lua`,"Order matters","If one Lua file depends on another, list the dependency first so it is available when later files load."],
  lua:["THE LOGIC","MyFirstAddon.lua","Lua files contain your addon’s behavior: listening for events, calling WoW APIs, storing data, and changing the interface.",`local addonName = ...\n\nlocal frame = CreateFrame("Frame")\nframe:RegisterEvent("PLAYER_LOGIN")\n\nframe:SetScript("OnEvent", function()\n  print(addonName .. " is ready!")\nend)`,"Keep the first version small","One file is enough for a first addon. Split code into modules only when each file has a clear responsibility."],
  xml:["OPTIONAL UI LAYOUT","MyFirstAddon.xml","XML can declare frames and visual templates. Many modern addons create their interface directly in Lua, so this file is optional.",`<Ui xmlns="http://www.blizzard.com/wow/ui/">\n  <Frame name="MyFirstAddonFrame">\n    <Size x="240" y="100"/>\n    <Anchors>\n      <Anchor point="CENTER"/>\n    </Anchors>\n  </Frame>\n</Ui>`,"Lua or XML?","Start with Lua unless a guide or an existing Blizzard template gives you a clear reason to use XML."],
  saved:["PERSISTENT SETTINGS","SavedVariables","SavedVariables are Lua tables named in the TOC. WoW saves them when you log out or reload and restores them next time.",`## SavedVariables: MyFirstAddonDB\n\n-- In MyFirstAddon.lua\nMyFirstAddonDB = MyFirstAddonDB or {\n  greeting = "Hello!",\n  enabled = true,\n}`,"Let WoW write the file","Do not edit the generated SavedVariables file while the game is running. Change the table through your addon code instead."]
};
const examples={
  login:{concept:"EVENTS + CHAT",title:"Say hello when you log in",description:"Create a frame, listen for the login event, and print a friendly message to the chat window.",challenge:'Include your character’s name using UnitName("player").',code:`local frame = CreateFrame("Frame")\nframe:RegisterEvent("PLAYER_LOGIN")\n\nframe:SetScript("OnEvent", function()\n  print("Hello, Azeroth!")\nend)`},
  slash:{concept:"SLASH COMMANDS",title:"Make your first chat command",description:"Register /hello and respond whenever the player types it into chat.",challenge:"Accept a name after the command, such as /hello Thrall.",code:`SLASH_MYFIRSTADDON1 = "/hello"\n\nSlashCmdList.MYFIRSTADDON = function(message)\n  if message == "" then\n    print("Hello from my addon!")\n  else\n    print("Hello, " .. message .. "!")\n  end\nend`},
  button:{concept:"FRAMES + SCRIPTS",title:"Put a button on the screen",description:"Create a movable-looking UI button, position it in the center, and react when it is clicked.",challenge:"Make the button draggable and remember its position between sessions.",code:`local button = CreateFrame("Button", nil, UIParent,\n  "UIPanelButtonTemplate")\nbutton:SetSize(140, 32)\nbutton:SetPoint("CENTER")\nbutton:SetText("Say hello")\n\nbutton:SetScript("OnClick", function()\n  print("You clicked it!")\nend)`}
};

function renderCategories(){
  $("#categoryRail").innerHTML=categories.map(([name,image])=>`<button class="category-button ${state.category===name?"active":""}" data-category="${name}" title="${name}"><img src="../../assets/ribbon-bar/${image}" alt=""><span>${name}</span></button>`).join("");
}
function filtered(){return addons.filter(a=>state.view!=="installed"||state.installed.has(a.id)).filter(a=>a.category===state.category).filter(a=>`${a.name} ${a.author} ${a.category}`.toLowerCase().includes(state.query));}
function render(){
  renderCategories(); const list=filtered();
  $("#addonGrid").innerHTML=list.length?list.slice(0,12).map(a=>{const yes=state.installed.has(a.id);return `<article class="addon-card"><div class="card-art"><img src="../../assets/ribbon-bar/${a.image}" alt=""><button class="${yes?"installed":""}" data-install="${a.id}">${yes?"Remove":"Install"}</button></div><div class="card-body"><h3>${a.name}</h3><p>by ${a.author}</p><div class="card-meta"><span>↓ ${a.downloads}</span><div class="card-links"><a href="https://github.com/" target="_blank" rel="noreferrer" aria-label="Open ${a.name} source project" title="View source project"><svg viewBox="0 0 24 24" aria-hidden="true"><circle cx="6" cy="6" r="2"></circle><circle cx="18" cy="6" r="2"></circle><circle cx="6" cy="18" r="2"></circle><path d="M6 8v8m2-8c2.5 0 3 4 6 4h2m2-4v8"></path></svg></a><button data-details="${a.id}">View details</button></div></div></div></article>`}).join(""):`<p class="empty">No addons found in this view.</p>`;
  $("#sectionEyebrow").textContent=state.view==="installed"?"YOUR COLLECTION":"COMMUNITY PICKS";
  $("#sectionTitle").textContent=state.view==="installed"?"My Addons":"Featured Addons";
}
function setView(view){
  state.view=view; document.querySelectorAll(".topnav").forEach(b=>b.classList.toggle("active",b.dataset.view===view));
  if(view==="loadouts"){showToast("Loadout builder preview coming next");state.view="discover";document.querySelector('[data-view="discover"]').classList.add("active");document.querySelector('[data-view="loadouts"]').classList.remove("active");}
  const workshop=state.view==="workshop";$("#catalogView").hidden=workshop;$("#workshopView").hidden=!workshop;
  if(!workshop)render();
}
function toggleInstall(id){
  const addon=addons.find(a=>a.id===id), installing=!state.installed.has(id);
  installing?state.installed.add(id):state.installed.delete(id);
  showToast(`${addon.name} ${installing?"installed":"removed"}`);render();
}
function details(id){
  const a=addons.find(x=>x.id===id),installed=state.installed.has(id),release=$("#release");
  $("#dialogIcon").src=`../../assets/ribbon-bar/${a.image}`;$("#dialogIcon").alt=`${a.category} icon`;
  $("#dialogCategory").textContent=a.category;$("#dialogTitle").textContent=a.name;$("#dialogAuthor").textContent=`by ${a.author}`;$("#dialogDescription").textContent=a.description;
  $("#dialogStatus").textContent=installed?"✓ INSTALLED":"AVAILABLE";$("#dialogVersion").textContent=`${1+(id%3)}.${id%10}.${(id*3)%10}`;$("#dialogSku").textContent=release.options[release.selectedIndex].text;$("#dialogDownloads").textContent=a.downloads;
  $("#dialogInstall").textContent=installed?"Remove Addon":"Install Addon";$("#dialogInstall").dataset.id=id;$("#detailsDialog").showModal();
}
function showToast(message){const t=$("#toast");t.textContent=message;t.classList.add("show");clearTimeout(toastTimer);toastTimer=setTimeout(()=>t.classList.remove("show"),2200);}
function applySku(sku){
  $(".launcher").dataset.sku=sku;
  const artwork=skuArtwork[sku],image=$("#skuArtwork");
  image.hidden=!artwork;
  if(artwork){image.src=artwork.src;image.alt=artwork.alt;}
}
function showWorkshopScreen(name){
  document.querySelectorAll("[data-workshop-screen]").forEach(button=>button.classList.toggle("active",button.dataset.workshopScreen===name));
  document.querySelectorAll("[data-workshop-panel]").forEach(panel=>panel.hidden=panel.dataset.workshopPanel!==name);
  $(".main-content").scrollTo({top:0,behavior:"smooth"});
}
function showFileLesson(name){
  const [eyebrow,title,description,code,noteTitle,note]=fileLessons[name];
  document.querySelectorAll("[data-file]").forEach(button=>button.classList.toggle("active",button.dataset.file===name));
  $("#fileExplainer").innerHTML=`<p class="eyebrow">${eyebrow}</p><h2>${title}</h2><p>${description}</p><pre><code>${code.replaceAll("&","&amp;").replaceAll("<","&lt;")}</code></pre><aside><strong>${noteTitle}</strong><span>${note}</span></aside>`;
}
function showExample(name){
  const example=examples[name];document.querySelectorAll("[data-example]").forEach(button=>button.classList.toggle("active",button.dataset.example===name));
  $("#exampleConcept").textContent=example.concept;$("#exampleTitle").textContent=example.title;$("#exampleDescription").textContent=example.description;$("#exampleChallenge").textContent=example.challenge;$("#exampleCode").textContent=example.code;
}

$("#categoryRail").addEventListener("click",e=>{const b=e.target.closest("[data-category]");if(b){state.category=b.dataset.category;render();}});
$("#addonGrid").addEventListener("click",e=>{const i=e.target.closest("[data-install]"),d=e.target.closest("[data-details]");if(i)toggleInstall(+i.dataset.install);if(d)details(+d.dataset.details);});
document.querySelectorAll(".topnav").forEach(b=>b.addEventListener("click",()=>setView(b.dataset.view)));
$("#search").addEventListener("input",e=>{state.query=e.target.value.toLowerCase();render();});
$("#updateButton").addEventListener("click",()=>{$("#updateButton").disabled=true;$("#updateStatus").textContent="Checking for updates…";setTimeout(()=>{$("#updateButton").disabled=false;$("#updateStatus").textContent="Last checked: just now";showToast("Everything is up to date");},900);});
$("#settingsButton").addEventListener("click",()=>showToast("Settings panel preview coming next"));
$("#learnSupport").addEventListener("click",()=>showToast("Donation and project links will appear on each addon's details page"));
$("#release").addEventListener("change",e=>{applySku(e.target.value);showToast(`Theme switched to ${e.target.options[e.target.selectedIndex].text}`);});
$(".dialog-close").addEventListener("click",()=>$("#detailsDialog").close());$("#detailsDialog").addEventListener("click",e=>{if(e.target.id==="detailsDialog")e.target.close();});
$("#dialogInstall").addEventListener("click",e=>{toggleInstall(+e.target.dataset.id);$("#detailsDialog").close();});
$("#dialogSupport").addEventListener("click",()=>showToast("Opening the author's preferred support page"));
$("#startWorkshop").addEventListener("click",()=>showWorkshopScreen("anatomy"));
document.querySelectorAll("[data-workshop-screen]").forEach(button=>button.addEventListener("click",()=>showWorkshopScreen(button.dataset.workshopScreen)));
document.querySelectorAll("[data-workshop-step]").forEach((button,index)=>button.addEventListener("click",()=>showWorkshopScreen(["anatomy","apis","examples"][index])));
document.querySelectorAll("[data-file]").forEach(button=>button.addEventListener("click",()=>showFileLesson(button.dataset.file)));
document.querySelectorAll("[data-example]").forEach(button=>button.addEventListener("click",()=>showExample(button.dataset.example)));
$("#copyExample").addEventListener("click",()=>{navigator.clipboard?.writeText($("#exampleCode").textContent);showToast("Example copied to clipboard");});
render();
