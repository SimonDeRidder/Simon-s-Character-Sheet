

function createEnum(values) {
	const enumObject = {};
	for (const val of values) {
		enumObject[val] = val;
	}
	return Object.freeze(enumObject);
}


function addElementNode(
	tag /*str*/,
	parent /*Node*/,
	text /*str*/,
	id /*str*/,
	className /*str*/,
	style=null /*Object*/,
) {
	let el = document.createElement(tag);
	if (text) {
		el.innerHTML = text;
	}
	if (id) {
		el.id = id;
	}
	if (className) {
		el.className = className;
	}
	if (style) {
		for(prop in style) {
			el.style[prop] = style[prop];
		}
	}
	parent.appendChild(el);
	return el;
}


function getAccessedFieldIds(code /*String*/) /*Set[String]*/ {
	const patterns = [/What\(([^\)]+)\)/g, /tdoc.getField\(([^\)]+)\)/g, /How\(([^\)]+)\)/g];

	function isStringLiteralString(theString /*String*/) /*boolean*/ {
		return (
			(theString.startsWith("'") && theString.endsWith("'") && !theString.slice(1, -1).includes("'"))
			|| (theString.startsWith('"') && theString.endsWith('"') && !theString.slice(1, -1).includes('"'))
		)
	}

	let all_matches = new Set();
	for (let pattern of patterns) {
		let matches = [...code.matchAll(pattern)];
		for (let match of matches) {
			if (isStringLiteralString(match[1].trim())) {
				let matchID = adapter_helper_convert_fieldname_to_id(match[1].trim().slice(1, -1));
				if (document.getElementById(matchID)) {
					all_matches.add(matchID);
				}
			} else {
				let fieldNameTokens = match[1].split("+");
				if (fieldNameTokens.length == 1) {
					// one token, not string
					throw "Variable encountered in matches for getAccessedFieldIds: " + match;
				} else {
					let pattern = "";
					let trimToken;
					for (let token of fieldNameTokens) {
						trimToken = token.trim();
						if (isStringLiteralString(trimToken)) {
							pattern += adapter_helper_convert_fieldname_to_id(trimToken.slice(1, -1));
						} else {
							// variable, could be anything
							// TODO: try to resolve
							if (trimToken.endsWith('Nmbr') || trimToken.endsWith('Num') || (trimToken == 'i')) {
								pattern += "\\d+";
							} else {
								pattern += ".+";
							}
						}
					}
					let fieldIDRegExp = new RegExp("^" + pattern + "$");
					[...document.getElementsByClassName('field')].forEach(element => {
						if (fieldIDRegExp.test(element.id)) {
							all_matches.add(element.id);
						}
					})
				}
			}
		}
	}
	// converted fields
	for (let abi of ['Cha', 'Str', 'Dex', 'Con', 'Wis', 'Int', 'HoS']) {
		let abi_matches = [...code.matchAll(/wasm_character.get_ability\('([^']+)'\)/g)];
		for (let match of abi_matches) {
			all_matches.add(match[1]);
		}
		let abi_mod_matches = [...code.matchAll(/wasm_character.get_ability_modifier\('([^']+)'\)/g)];
		for (let match of abi_mod_matches) {
			all_matches.add(match[1] + "_Mod");
		}
		let level_matches = [...code.matchAll(/wasm_character.get_level\(\)/g)];
		if (level_matches) {
			all_matches.add("Character_Level")
		}
	}
	return all_matches;
}
