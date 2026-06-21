
// TODO: connect "Color.Theme"
// TODO: "SaveIMG.Header.Left." + colour, "SaveIMG.Divider." + colour
// TODO: connect "Extra.Layers Button", "Buttons"
// TODO: connect thermoM to floating fade-out status message
// TODO: make sure field getters in CurrentEvals.hp trigger calculation of (Comp.Use.)HP.Max
// TODO: set default values up to P4.AScomp.Comp.Type
// TODO: parse CurrentEvals stuff for proper event triggering

// Load functions

function loadScript(path /*str*/) /*Promise*/ {
	return new Promise(function (resolve, reject) {
		const scriptElement = document.createElement("script");
		scriptElement.src = path;
		scriptElement.onload = () => resolve(scriptElement);
		document.body.appendChild(scriptElement);
	});
}

function makeSaveLoadButtons() {
	return new Promise(function (resolve, reject) {
		app.addToolButton({
			cName: "save",
			oIcon: null,
			cExec: "adapter_helper_save_all()",
			cLabel: "Save"
		});
		app.addToolButton({
			cName: "load",
			oIcon: null,
			cExec: "adapter_helper_load()",
			cLabel: "Load"
		});
		resolve();
	});
}

function initialCalculationEvents() {
	// First set CurrentWeapons (errors otherwise)
	FindWeapons();
	FindCompWeapons();
	// trigger ability mods calculation (and ST calculation) and AC_Dexterity_Modifier calculation (and all skills) (and Attack.i.To Hit)
	eventManager.handle_event(EventType.Con_change);
	eventManager.handle_event(EventType.Cha_change);
	eventManager.handle_event(EventType.Dex_change);
	eventManager.handle_event(EventType.Int_change);
	eventManager.handle_event(EventType.HoS_change);
	eventManager.handle_event(EventType.Str_change);
	eventManager.handle_event(EventType.Wis_change);
	eventManager.handle_event(EventType.Con_Mod_change);
	eventManager.handle_event(EventType.Cha_Mod_change);
	eventManager.handle_event(EventType.Dex_Mod_change);
	eventManager.handle_event(EventType.Int_Mod_change);
	eventManager.handle_event(EventType.HoS_Mod_change);
	eventManager.handle_event(EventType.Str_Mod_change);
	eventManager.handle_event(EventType.Wis_Mod_change);
	// trigger Proficiency_Bonus calculation
	document.getElementById('Proficiency_Bonus_Modifier').dispatchEvent(new Event('change'));
	// trigger weight texts
	document.getElementById('Unit_System').dispatchEvent(new Event('change'));
	// trigger feat_name & magic items calculations
	for (let i = 1; i <= 8; i++) {
		document.getElementById('Feat_Name_' + String(i)).dispatchEvent(new Event('calculate'));
	}
	for (let i = 1; i <= 12; i++) {
		document.getElementById('Extra.Magic_Item_' + String(i)).dispatchEvent(new Event('calculate'));
	}
	// trigger AC calculation
	document.getElementById('AC_Armor_Bonus').dispatchEvent(new Event('change'));
	// trigger Adventuring_Gear_Location_Subtotal_* and Adventuring_Gear_Weight_Subtotal_* calculation
	document.getElementById('Adventuring_Gear_Amount_1').dispatchEvent(new Event('change'));
	document.getElementById('Adventuring_Gear_Amount_19').dispatchEvent(new Event('change'));
	document.getElementById('Adventuring_Gear_Amount_37').dispatchEvent(new Event('change'));
	document.getElementById('Extra.Gear_Amount_1.1').dispatchEvent(new Event('change'));
	document.getElementById('P4.AScomp.Comp.eqp.Gear_Amount_1').dispatchEvent(new Event('change'));
}

function setSheetVersion() {
	return new Promise(function (resolve, reject) {
		for (let sheetInfoElement of document.getElementsByClassName("sheetinfo")) {
			sheetInfoElement.innerHTML = "Simon&apos;s D&amp;D 5e Character Sheet " + this.info.version + " (MPMB " + this.info.SheetVersion + ")";
		}
		resolve();
	});
}

async function fetchFixedAdditionalScripts() {
	for (let item of []) {
		let fileName = item[0];
		let scriptName = item[1];
		console.log("adding additional scripts from", scriptName);
		let file_response = await fetch("additional content/" + fileName);
		let file_content = await file_response.text();
		CurrentScriptFiles[scriptName] = file_content;
	}
}


async function loadAll() {
	await loadScript('_functions/DomParser.js')
		.then(script => loadScript('_functions/Functions0.js'))
		.then(script => loadScript('import_utils/overwrite_Functions0.js'))
		.then(script => loadScript('_functions/Functions1.js'))
		.then(script => loadScript('_functions/Functions2.js'))
		.then(script => loadScript('_functions/Functions3.js'))
		.then(script => loadScript('_functions/FunctionsImport.js'))
		.then(script => loadScript('_functions/FunctionsResources.js'))
		.then(script => loadScript('_functions/FunctionsSpells.js'))
		.then(script => loadScript('_functions/Shutdown.js'))
		.then(script => loadScript('_variables/Icons.js'))
		.then(script => loadScript('_variables/Lists.js'))
		.then(script => loadScript('_variables/ListsBackgrounds.js'))
		.then(script => loadScript('_variables/ListsClasses.js'))
		.then(script => loadScript('_variables/ListsCompanions.js'))
		.then(script => loadScript('_variables/ListsCreatures.js'))
		.then(script => loadScript('_variables/ListsFeats.js'))
		.then(script => loadScript('_variables/ListsGear.js'))
		.then(script => loadScript('_variables/ListsMagicItems.js'))
		.then(script => loadScript('_variables/ListsPsionics.js'))
		.then(script => loadScript('_variables/ListsRaces.js'))
		.then(script => loadScript('_variables/ListsSources.js'))
		.then(script => loadScript('_variables/ListsSpells.js'))
		.then(script => makeSaveLoadButtons())
		.then(script => setSheetVersion())
		.then(script => loadScript('_functions/Startup.js'));
}


function startLoadingContent() {
	if (navigator.userAgent.indexOf("Firefox") > 0) {
		// For some reason, Firefox needs just a little extra delay to get things loaded
		setTimeout(loadAll, 100);
	} else {
		loadAll();
	}
}


if (document.readyState === "loading") {
	// Loading hasn't finished yet
	document.addEventListener("DOMContentLoaded", startLoadingContent);
} else {
	// `DOMContentLoaded` has already fired
	startLoadingContent();
}
