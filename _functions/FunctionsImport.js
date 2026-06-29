// Run the custom defined user scripts, if any exist
function RunUserScript(atStartup) {
	if (atStartup) tDoc.noDeprecatedWarnings = []; // make this globally accessible
	var ScriptsAtEnd = [], ScriptAtEnd = [];
	var minSheetVersion = [0, ""], maxSheetVersion = [999999, ""];
	var RunFunctionAtEnd = function(inFunction) { // not unused (eval in scripts)
		if (inFunction && typeof inFunction === "function") ScriptAtEnd.push(inFunction);
	};
	var runIt = function(aScript, scriptName) {
		var RequiredSheetVersion = function(minNumber, maxNumber) { // not unused (eval in scripts)
			if (atStartup) return;
			var getVersString = function(input) {
				var inputStr = input.toString();
				return /-|beta|\+/i.test(inputStr) ? inputStr.replace(/^\D+/, "").replace(/([^\-])\.?beta/i, "$1-beta") : getSemVers(input);
			}
			var minSemVers = getVersString(minNumber);
			var testMinNmbr = semVersToNmbr(minSemVers);
			if (testMinNmbr > minSheetVersion[0]) minSheetVersion = [testMinNmbr, minSemVers];
			if (maxNumber) {
				var maxSemVers = getVersString(maxNumber);
				var testMaxNmbr = semVersToNmbr(maxSemVers);
				if (testMaxNmbr < maxSheetVersion[0]) maxSheetVersion = [testMaxNmbr, maxSemVers];
			}
		};
		try {
			IsNotUserScript = false;
			ScriptAtEnd = [];
			minSheetVersion = [0, ""];
			maxSheetVersion = [9E9, ""];
			eval(aScript);
			IsNotUserScript = true;
			if (ScriptAtEnd.length > 0) ScriptsAtEnd = ScriptsAtEnd.concat(ScriptAtEnd);
			var failedTestMsg;
			if (sheetVersion < minSheetVersion[0]) {
				failedTestMsg = {
					cMsg: 'The add-on script "' + scriptName + '" reports that it requires at least version number v' + minSheetVersion[1] + ", and is thus probably not compatible with the version of the sheet that you are using (which is v" + semVers + ').'+
						(sheetVersion >= maxSheetVersion[0] ? "\nThe add-on script also has a maximum version requirement. Adding it to a v" + maxSheetVersion[0] + " or higher sheet will result in this same error message." : '')+
						(minSheetVersion[0] >= 24000000 ? '' : '\nThis could be because from v24.0.0 onwards, the sheet uses the 5.5e (2024) rules, while lower versions use the 5e (2014) rules.')+
						'\n\nDo you want to continue using this add-on script in the sheet? If you select NO, the "' + scriptName + '" add-on script will be skipped and removed.'+
						'\n\nYou can find other versions of the sheet with the "Get Latest Version" bookmark.',
					nIcon: 2,
					cTitle: "Add-on script was made for " + (sheetVersion >= maxSheetVersion[0] ? "another" : "newer") + " version of the PDF!",
					nType: 2,
				};
			} else if (sheetVersion >= maxSheetVersion[0]) {
				failedTestMsg = {
					cMsg: 'The add-on script "' + scriptName + '" reports that it requires a sheet with a version number lower than v' + maxSheetVersion[1] + ", and is thus probably not compatible with the version of the sheet that you are using (which is v" + semVers + ').'+
						'\n\nDo you want to continue using this add-on script in the sheet? If you select NO, the "' + scriptName + '" add-on script will be skipped and removed.'+
						'\n\nYou can find other versions of the sheet with the "Get Latest Version" bookmark.',
					nIcon: 2,
					cTitle: "Add-on script was made for lower version of the PDF!",
					nType: 2,
				};
			}
			if (failedTestMsg && app.alert(failedTestMsg) !== 4) return false;
			return true;
		} catch (error) {
			if (/out of memory/i.test(error.toSource())) return "outOfMemory";
			IsNotUserScript = true;
			var forNewerVersion = sheetVersion < minSheetVersion[0];
			var forOlderVersion = sheetVersion > maxSheetVersion[0];
			var eText = "The add-on script " + '"' + scriptName + '"';
			if (forNewerVersion || forOlderVersion) {
				if (forNewerVersion) eText += " reports that it requires at minimum v" + minSheetVersion[1];
				if (forNewerVersion && forOlderVersion) {
					eText += " and at maximum v" + maxSheetVersion[1];
				} else if (forOlderVersion) {
					eText += " reports that it requires at maximum v" + maxSheetVersion[1];
				}
				eText += " of MPMB's Character Sheet. However, the version of the sheet that you are using is v" + semVers + ", which is probably why ";
			} else {
				eText += " is faulty, ";
			}
			eText += 'it returns the following error when run:\n\t"' + error;
			if (typeof error === "object") for (var e in error) eText += "\n\t  " + e + ": " + error[e];
			eText += '"\n\n';
			if (forNewerVersion || forOlderVersion) eText += 'This could be because from v24.0.0 onwards, the sheet uses the 5.5e (2024) rules, while lower versions use the 5e (2014) rules.\n\n';
			eText += "The add-on script has been removed from this pdf.";
			eText += "\n\nFor a more specific error and one that includes the error's line number, try running the add-on script from the JavaScript Console.";
			if (!forNewerVersion && !forOlderVersion) eText += "\n\nPlease contact the add-on script's author to report this issue.";
			app.alert({
				cMsg: eText,
				nIcon: 0,
				cTitle: forNewerVersion || forOlderVersion ? "Add-on script was made for another version!" : "Error in running user add-on script"
			});
			return false;
		};
	};

	// first run the code added by importing whole file(s)
	var scriptsResult = true;
	for (var iScript in CurrentScriptFiles) {
		var runIScript = runIt(CurrentScriptFiles[iScript], iScript);
		if (!runIScript) {
			delete CurrentScriptFiles[iScript];
			scriptsResult = runIScript;
		} else if (runIScript == "outOfMemory") {
			break;
		}
	};

	// run the functions that are meant to be saved till the very end of all the scripts
	if (ScriptsAtEnd.length > 0) {
		var functionErrors = [];
		IsNotUserScript = false;
		for (var i = 0; i < ScriptsAtEnd.length; i++) {
			try { ScriptsAtEnd[i](); } catch (error) {
				functionErrors.push('The function starting with "' + ScriptsAtEnd[i].toString().slice(0,100) + '"\nresulted in the error: "' + error + '"');
			};
		};
		IsNotUserScript = true;
		if (!atStartup && functionErrors.length > 0) {
			app.alert({
				cMsg : "One or more of the script you entered has a 'RunFunctionAtEnd()' statement. One or more of those functions gave an error. The sheet can't tell you which of those gave an error exactly, but it can tell you what the errors are:\n\n" + functionErrors.join("\n\n"),
				nIcon : 0,
				cTitle : "Error in RunFunctionAtEnd() from user script(s)"
			});
		};
	};

	// fix wrong reference (common mistake when adding classes)
	fixClassReferences();

	// when run at startup and one of the script fails, update all the dropdowns
	if (runIScript == "outOfMemory") {
		outOfMemoryErrorHandling(atStartup);
	} else if (atStartup) {
		UpdateDropdown("resources");
	} else { // i.e. run to test file import with RunUserScript(false, false);
		return scriptsResult;
	};
};

// Fix a common mistake in adding classes, having subclass references that don't work
function fixClassReferences(bDontKickSubclasses) {
	// Loop through all classes
	for (var sClass in ClassList) {
		var oClass = ClassList[sClass];
		// If the subclasses attribute doesn't exist or is malformed, fix it
		if (!oClass.subclasses || !isArray(oClass.subclasses) || !isArray(oClass.subclasses[1])) {
			oClass.subclasses = [
				oClass.subclasses[0] && typeof oClass.subclasses[0] === "string" ? oClass.subclasses[0] : oClass.name + " Subclass",
				[]
			];
		} else if (!bDontKickSubclasses) {
			// Loop through all the subclasses from end to start and delete any that don't exist in the ClassSubList object and any duplicates
			var arrDupl = [];
			for (var i = oClass.subclasses[1].length - 1; i >= 0; i--) {
				var sSubcl = oClass.subclasses[1][i];
				if (!ClassSubList[sSubcl] || arrDupl.indexOf(sSubcl) !== -1) {
					oClass.subclasses[1].splice(i, 1);
					displayError(false, 'The subclass "' + sSubcl + '" for the class "' + oClass.name + "\" is missing from the ClassSubList object, or appears multiple times in the `subclasses` attribute. Please contact its author to have this issue corrected. The subclass will be ignored for now.\nBe aware that if you add a subclass using the `AddSubClass()` function, you shouldn't list it in the `subclasses` attribute, the function will take care of that.");
				} else {
					arrDupl.push(sSubcl);
				}
			}
		}
		// Create subclassGainedLevel attribute if it doesn't exist
		if (!oClass.subclassGainedLevel) {
			oClass.subclassGainedLevel = 3; // set it to level 3 by default
			for (var key in oClass.features) {
				if (key.indexOf("subclassfeature") !== -1 && oClass.features[key].minlevel && oClass.features[key].minlevel < oClass.subclassGainedLevel) {
					oClass.subclassGainedLevel = oClass.features[key].minlevel;
				}
			}
		}
	}
}

// Define some custom import script functions as document-level functions so custom scripts including these can still be run from console
function RequiredSheetVersion(minNumber, maxNumber) {
	var getVersString = function(input) {
		var inputStr = input.toString();
		return /-|beta|\+/i.test(inputStr) ? inputStr.replace(/^\D+/, "").replace(/([^\-])\.?beta/i, "$1-beta") : getSemVers(input);
	}
	var minSemVers = getVersString(minNumber);
	var testMinNmbr = semVersToNmbr(minSemVers);
	var testMax = false, maxSemVers, testMaxNmbr;
	if (maxNumber) {
		var maxSemVers = getVersString(maxNumber);
		var testMaxNmbr = semVersToNmbr(maxSemVers);
		testMax = true;
	}
	if (sheetVersion < testMinNmbr || (testMax && sheetVersion >= testMaxNmbr)) {
		app.alert({
			cMsg: "The RequiredSheetVersion() function in your script suggests that the script is made for another version of the MPMB's Character Sheet, minimally requiring v" + minSemVers + (testMax ? ", but lower than v" + maxSemVers : "") + "."+
				"\nThis current PDF is v" + semVers + " and will likely not work properly."+
				"\nAlternatively, you might not be using the RequiredSheetVersion() function correctly.",
			nIcon: 2,
			cTitle: "Script was made for another version of the PDF!",
		});
	}
};
function RunFunctionAtEnd(inFunc) {
	if (!inFunc && typeof inFunc !== "function") return;
	var funcstart = inFunc.toString().replace(/function *\([^)]*\) *{(\r\n)*\t*/i,"").substr(0,50);
	app.alert({
		cMsg : "The script you are running from the console contains the function RunFunctionAtEnd(). This function can be exectured from the console, but will be executed immediately after you close this dialog, and not at the end of all the code you are trying to run from console. When you import this script as a file, or manually paste it into the dialog for scripts, it will be run at the end of all scripts as intended.\n\nAfter clicking 'OK', the function will be run that starts with the following:\n\t\"" + funcstart + "...\"",
		nIcon : 1,
		cTitle : "RunFunctionAtEnd() works different when executed from the console"
	});
	try {
		inFunc();
	} catch(e) {
		app.alert({
			cMsg : "The function entered in 'RunFunctionAtEnd()', that starts with:\n\t\"" + funcstart + "...\"\nproduces the following error, which might be because it was executed from the console:\n\n" + e,
			nIcon : 0,
			cTitle : "Error in RunFunctionAtEnd() from user script(s)"
		});
	};
};

// a way to add a racial variant without conflicts
function AddRacialVariant(race, variantName, variantObj) {
	race = race.toLowerCase();
	variantName = variantName.toLowerCase();
	if (!RaceList[race]) return;
	if (!RaceList[race].variants || !isArray(RaceList[race].variants)) RaceList[race].variants = [];
	var suffix = 1;
	while (RaceList[race].variants.indexOf(variantName) !== -1) {
		suffix += 1;
		variantName += suffix;
	};
	RaceList[race].variants.push(variantName);
	RaceSubList[race + "-" + variantName] = variantObj;
};

// a way to add a subclass without conflicts
function AddSubClass(iClass, subclassName, subclassObj) {
	iClass = iClass.toLowerCase();
	subclassName = subclassName.toLowerCase();
	if (!ClassList[iClass]) return;
	if (!ClassList[iClass].subclassGainedLevel || !ClassList[iClass].subclasses) fixClassReferences(true);
	var suffix = 1;
	var baseScNm = iClass + "-" + subclassName;
	var fullScNm = baseScNm;
	while (ClassList[iClass].subclasses[1].indexOf(fullScNm) !== -1 || ClassSubList[fullScNm]) {
		suffix++;
		fullScNm = baseScNm + "-" + suffix;
	};
	ClassList[iClass].subclasses[1].push(fullScNm);
	ClassSubList[fullScNm] = subclassObj;
	return fullScNm;
};

// a way to add a background variant without conflicts
function AddBackgroundVariant(background, variantName, variantObj) {
	background = background.toLowerCase();
	variantName = variantName.toLowerCase();
	if (!BackgroundList[background]) return;
	if (!BackgroundList[background].variant || !isArray(BackgroundList[background].variant)) BackgroundList[background].variant = [];
	var suffix = 1;
	var fullBvNm = background + "-" + variantName;
	while (BackgroundList[background].variant.indexOf(fullBvNm) !== -1) {
		suffix += 1;
		fullBvNm += suffix;
	};
	BackgroundList[background].variant.push(fullBvNm);
	BackgroundSubList[fullBvNm] = variantObj;
};

// A way to add an (extra)choice to a class feature / racial feature / feat / magic item
/* Input Valiables Definition
	pObj    parent object, e.g. ClassList.warlock.features["eldritch invocations"]
	cType   type of choice, false for `choice`, true for `extrachoice`
	cName   name of the choice as it will appear in the menu (with capitalisation)
	cObj    the choice object
	force   if != false, force creation of the (extra)choices array
	        if cType == true, use the force string for the extraname
	bSort	if != false sort the array after the choice was added
			Not for class features, where (extra)choices arrays are sorted before displaying the menu,
			but good for magic items, where the arrays are never sorted automatically.
*/
function AddFeatureChoice(pObj, cType, cName, cObj, force, bSort) {
	if (!pObj) return; // parent object doesn't exist
	var aObj = pObj; // the object where the (extra)choice will be added to
	var cNameLC = cName.toLowerCase();
	cType = cType ? "extrachoices" : "choices";
	if (!pObj[cType]) { // choice array doesn't exist
		if (!force) return; // no choice array and not forced, so quit now
		if (cType === "extrachoices" && typeof force == "string") {
			FixAutoSelForceChoices(pObj);
			if (pObj.choiceSetsExtrachoices) {
				pObj.extrachoicesRemember = [];
			}
			if (pObj.choices && pObj.defaultChoice) {
				pObj.choiceSetsExtrachoices = true;
				aObj = pObj[pObj.defaultChoice];
			}
			if (!aObj.extraname) aObj.extraname = force;
		}
		aObj[cType] = [];
	}
	// Stop if adding something that already exists, so no reason to continue
	if (aObj[cNameLC] && aObj[cNameLC].toSource() == cObj.toSource()) return;
	// when adding a new choice that contains extrachoices of its own
	if (cType === "choices") {
		if (cObj.extrachoices) {
			// copy the extrachoices for remembering the original value, if any
			if (pObj.extrachoices && !pObj.extrachoicesRemember) {
				pObj.extrachoicesRemember = pObj.extrachoices;
				pObj.extranameRemember = pObj.extraname;
				pObj.extraTimesRemember = pObj.extraTimes;
			}
			pObj.choiceSetsExtrachoices = true;
		}
		// also do something if it contains autoSelectExtrachoices
		if (cObj.autoSelectExtrachoices) {
			if (pObj.autoSelectExtrachoices && !pObj.autoSelectExtrachoicesRemember) {
				pObj.autoSelectExtrachoicesRemember = pObj.autoSelectExtrachoices;
			}
			FixAutoSelForceChoices(pObj, false, cObj);
		}
	}
	// See if something by its name already exists and amend it, if so
	var useName = cName;
	var suffix = 1;
	while (aObj[cType].indexOf(useName) !== -1 || aObj[useName.toLowerCase()]) {
		suffix += 1;
		useName = cName + " [" + suffix + "]";
	};
	// Add the new (extra)choice
	aObj[cType].push(useName);
	if (bSort) aObj[cType].sort();
	if (cType === "extrachoices" && aObj.extrachoicesRemember) pObj.extrachoicesRemember.push(useName);
	aObj[useName.toLowerCase()] = cObj;
}
// --- backwards compatibility --- //
function AddWarlockInvocation(invocName, invocObj) { // Add a warlock invocation
	AddFeatureChoice(ClassList.warlock.features["eldritch invocations"], true, invocName, invocObj);
};
function AddWarlockPactBoon(boonName, boonObj) { // Add a warlock pact boon
	AddFeatureChoice(ClassList.warlock.features["pact boon"], false, boonName, boonObj);
};

// a way to add fighting styles to multiple classes; fsName is how it will appear in the menu
function AddFightingStyle(classArr, fsName, fsObj) {
	if (classArr.indexOf("ranger") !== -1 && classArr.indexOf("rangerua") == -1 && ClassList["rangerua"]) classArr.push("rangerua");
	for (var i = 0; i < classArr.length; i++) {
		var aClass = ClassList[classArr[i]];
		var sClass = ClassSubList[classArr[i]];
		if (aClass) {
			AddFeatureChoice(aClass.features["fighting style"], false, fsName, fsObj);
			if (classArr[i] === "fighter" && ClassSubList["fighter-champion"]) {
				AddFeatureChoice(ClassSubList["fighter-champion"].features["subclassfeature10"], false, fsName, fsObj);
			}
		} else if (sClass) {
			for (var clFea in sClass.features) {
				var sFea = sClass.features[clFea];
				if (sFea.choices && (/^(?=.*fighting)(?=.*style).*$/i).test(sFea.name)) {
					AddFeatureChoice(sClass.features[clFea], false, fsName, fsObj);
				}
			}
		}
	};
};

// make an existing class feature into a feature with choices, and add the original as a default choice
function CreateClassFeatureVariant(clName, clFea, varName, varObj) {
	if (ClassList[clName] && ClassList[clName].features[clFea]) {
		var aFea = ClassList[clName].features;
	} else if (ClassSubList[clName] && ClassSubList[clName].features[clFea]) {
		var aFea = ClassSubList[clName].features;
	} else {
		return;
	}
	if (!aFea[clFea].choices) {
		// Create a new choice system, with the 'normal' feature as a choice that is selected by default
		var origFea = newObj(aFea[clFea]);
		var choiceNm = "[original] " + origFea.name;
		var choiceNmLC = choiceNm.toLowerCase();
		aFea[clFea] = {
			name : origFea.name + " or a Variant",
			source: origFea.source,
			minlevel : origFea.minlevel,
			description : '\n   Select ' + origFea.name + ' or a variant using the "Choose Feature" button above',
			choices : [choiceNm],
			defaultChoice : choiceNmLC,
			choiceSetsExtrachoices : origFea.extrachoices ? true : false
		}
		aFea[clFea][choiceNmLC] = origFea;
		if (origFea.autoSelectExtrachoices) {
			aFea[clFea].autoSelectExtrachoices = origFea.autoSelectExtrachoices;
			FixAutoSelForceChoices(aFea[clFea], origFea.extraname, origFea);
		}
		if (origFea.extrachoices) {	
			// add the extrachoices offered in the choice to the parent object
			for (var i = 0; i < origFea.extrachoices.length; i++) {
				var xtrStr = origFea.extrachoices[i].toLowerCase();
				if (origFea[xtrStr]) aFea[clFea][xtrStr] = origFea[xtrStr];
			}
		}
	}
	AddFeatureChoice(aFea[clFea], false, varName, varObj);
}

// Fix autoSelectExtrachoices
function FixAutoSelForceChoices(pObj, sExtraname, cObj) {
	if (!pObj.autoSelectExtrachoices) return;
	if (!isArray(pObj.autoSelectExtrachoices)) pObj.autoSelectExtrachoices = [pObj.autoSelectExtrachoices];
	for (var i = 0; i < pObj.autoSelectExtrachoices.length; i++) {
		var aObj = pObj.autoSelectExtrachoices[i];
		if (!aObj || !aObj.extrachoice) continue;
		// make sure the parent object has the extrachoice as an attribute
		if (cObj && cObj[aObj.extrachoice] && !pObj[aObj.extrachoice]) {
			pObj[aObj.extrachoice] = cObj[aObj.extrachoice];
		} else if (!pObj[aObj.extrachoice]) {
			continue;
		}
		// force the extraname per object, so it is never taken from the parent object
		if (!pObj[aObj.extrachoice].extraname && !aObj.extraname) {
			aObj.extraname = sExtraname ? sExtraname : pObj.extraname;
		}
	}
}
