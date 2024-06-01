//functions to call at startup (in the right order)
async function InitializeEverything(classes, primaryClass) {
	calcStop();
	GetStringifieds(); //populate some variables stored in fields

	// Define some document level variables before and after running the user scripts
	InitiateLists();
	await fetchFixedAdditionalScripts();
	RunUserScript(true);
	spellsAfterUserScripts();

	if (!minVer) {
		SetGearVariables();
		setListsUnitSystem(false, true);
		await getDynamicFindVariables({}, "", classes, primaryClass);
		UpdateTooSkill();
		SetRichTextFields();
		MakeAdventureLeagueMenu();
	};

	SetHighlighting();
	MakeButtons();
	wasm_character.stop_startup();
	await calcCont(true);
	tDoc.dirty = false; //reset the dirty status, so the user is not asked to save without there having been any changes made
	app.loaded = true;
}

InitializeEverything({}, "");
