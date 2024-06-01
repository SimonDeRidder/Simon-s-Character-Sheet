//recursively merge two objects and return the result
function MergeRecursive(obj1, obj2) {
	for (var p in obj2) {
		try {
			// Property in destination object set; update its value.
			if (obj2[p].constructor == Object) {
				obj1[p] = MergeRecursive(obj1[p], obj2[p]);
			} else {
				obj1[p] = obj2[p];
			}
		} catch (e) {
			// Property in destination object not set; create it and set its value.
			obj1[p] = obj2[p];
		}
	}
	return obj1;
};

//recursively move single element from object1 into object2
function MergeElement(obj1, elem, obj2) {
	obj2 = SetNegativeElement(obj2);
	for (var p in obj1) {
		// Property in destination object set; update its value.
		if (obj1[p].constructor == Object && p !== elem) {
			if (!obj2[p]) obj2[p] = {};
			var newObj = MergeElement(obj1[p], elem, obj2[p]);
			obj1[p] = newObj[0];
			obj2[p] = newObj[1];
		} else if (p === elem) {
			if (obj1[p].constructor == Object && obj2[p]) {
				obj2[p] = MergeRecursive(obj2[p], obj1[p]);
			} else {
				obj2[p] = obj1[p];
			}
			delete obj1[p];
		}
	}
	return [CleanObject(obj1), CleanObject(obj2)];
};

//recursively remove elements of an object that are themselves empty objects
function CleanObject(obj) {
	for (var p in obj) {
		if (obj[p].constructor == Object) {
			for (var q in obj[p]) {
				if (obj[p][q].constructor == Object) {
					CleanObject(obj[p]);
				}
			}
			if (!ObjLength(obj[p])) delete obj[p];
		}
	}
	return obj;
};

//reduce an object to just the nodes
function ReduceObject(obj) {
	var hasSub = false;
	for (var p in obj) {
		if (obj[p].constructor == Object) {
			hasSub = true;
			obj[p] = ReduceObject(obj[p]);
			var emptyObj = true;
			for (var q in obj[p]) {
				emptyObj = false;
				break;
			}
			if (emptyObj) obj[p] = -1;
		} else {
			delete obj[p];
		}
	}
	return hasSub ? obj : false;
};

//get the positive value of a (sub)object
function GetPositiveElement(obj) {
	var thePos = false;
	for (var p in obj) {
		if (obj[p].constructor == Object) {
			thePos = GetPositiveElement(obj[p]);
		} else if (obj[p] > 0) {
			thePos = p;
		}
		if (thePos !== false) break;
	}
	return thePos;
};

//Set the negative value for all (sub)objects
function SetNegativeElement(obj) {
	for (var p in obj) {
		if (obj[p].constructor == Object) {
			obj[p] = SetNegativeElement(obj[p]);
		} else if (obj[p] > 0) {
			obj[p] = -1 * obj[p];
		}
	}
	return obj;
};

//make an array of an object, with the option to include just the nodes (type === "nodes"), just the elements (type === "elements"), or all (type === "all")
function ObjectToArray(obj, type, testObj) {
	var theArr = [];
	var testArr = testObj && isArray(testObj) ? testObj : (testObj ? ObjectToArray(testObj, "nodes") : []);
	for (var p in obj) {
		if (obj[p].constructor == Object) {
			theArr.push.apply(theArr, ObjectToArray(obj[p], type, testObj));
			if ((/nodes|all/i).test(type) && testArr.indexOf(p) === -1) theArr.push(p);
		} else if ((/elements|all/i).test(type) && testArr.indexOf(p.replace(/^ basic /, "")) === -1) {
			theArr.push(p);
		}
	}
	return theArr;
};

//a function to test if the input is not being excluded by the resource dialog (returns true if excluded)
function testSource(key, obj, CSatt, concise) {
	if (!obj) return true;
	if (!obj.source && (!CSatt || CSatt == "classExcl")) return false;
	var theRe = false;
	var tSrc = !obj.source && CSatt && CSatt !== "classExcl" ? false : parseSource(obj.source);
	if (tSrc && tSrc.length > 0) {
		var srcExcluded = function(srcObj) {
			return !SourceList[srcObj[0]] || CurrentSources.globalExcl.indexOf(srcObj[0]) !== -1;
		};
		var isExcl = tSrc.every(srcExcluded);
		theRe = isExcl && concise ? "source" : isExcl;
	};
	if (!theRe && CSatt && CurrentSources[CSatt]) {
		theRe = CurrentSources[CSatt].indexOf(key) !== -1;
		if (!theRe && obj.choices && CSatt !== "classExcl") {
			var exclChoices = 0;
			for (var i = 0; i < obj.choices.length; i++) {
				var aCh = obj.choices[i].toLowerCase();
				if (!obj[aCh] || CurrentSources[CSatt].indexOf(key + "-" + aCh) !== -1) exclChoices++;
			}
			if (obj.choices.length == exclChoices) theRe = true;
		}
	} 
	return theRe;
};

//a function to make the source attribute into a consistent array [[source, page]] and move excluded sources to the end
function parseSource(srcObj) {
	if (!srcObj) return false;
	var uObj = false;
	if (typeof srcObj == "string") {
		if (SourceList[srcObj]) uObj = [[srcObj, 0]];
	} else if (srcObj.length === 2 && typeof srcObj[0] == "string" && !isArray(srcObj[1])) {
		if (SourceList[srcObj[0]]) uObj = [srcObj];
	} else if (srcObj.length === 1 && !isArray(srcObj[0])) {
		if (SourceList[srcObj[0]]) uObj = [[srcObj[0], 0]];
	} else {
		uObj = srcObj;
	}
	if (!uObj) return false;
	var theRe = [];
	var theSRD = [];
	var areExcl = [];
	for (var i = 0; i < uObj.length; i++) {
		var toUse = !isArray(uObj[i]) ? [uObj[i], 0] : uObj[i].length > 1 ? uObj[i] : [uObj[i][0], 0];
		if (!toUse[0] || !SourceList[toUse[0]]) {
			continue;
		} else if (uObj[i][0] === "SRD") {
			theSRD.push(toUse);
		} else if (CurrentSources.globalExcl.indexOf(toUse[0]) !== -1) {
			areExcl.push(toUse);
		} else {
			theRe.push(toUse);
		};
	};
	return theRe.concat(theSRD).concat(areExcl);
};

//a function to make a readable string of the source
// verbosity = full (full source name), abbr (source abbreviation), page (, page), first (only first one found that is included), multi (add line break after each entry)
function stringSource(obj, verbosity, prefix, suffix) {
	var theSrc = parseSource(obj.source);
	if (theSrc) {
		var theRe = "";
		verbosity = verbosity.toLowerCase();
		var sFull = verbosity.indexOf("full") !== -1;
		var pFull = verbosity.indexOf("page") !== -1;
		for (var i = 0; i < theSrc.length; i++) {
			if (CurrentSources.globalExcl.indexOf(theSrc[i][0]) !== -1) continue;
			if (theRe) theRe += !pFull ? ", " : verbosity.indexOf("multi") !== -1 ? ";\n" : "; ";
			theRe += sFull ? SourceList[theSrc[i][0]].name : SourceList[theSrc[i][0]].abbreviation;
			theRe += !theSrc[i][1] ? "" : (pFull ? ", page " : " ") + theSrc[i][1];
			if (verbosity.indexOf("first") !== -1) break;
		};
		if (theRe && theRe.indexOf("\n") !== -1) theRe += ".";
		return theRe ? (prefix ? prefix : "") + theRe + (suffix ? suffix : "") : "";
	} else {
		return "";
	};
};
