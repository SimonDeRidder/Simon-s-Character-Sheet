// TODO: make map from 'name' to more unique element id
const modalDialogID = "modalDialog";
const modalCanvasID = "modalCanvas";
const modalDialogButtonLinkClass = "modalDialogButtonLink";

const dialogManager = {
	createDialog: async function (
		title /*str*/,
		body /*[Object]*/,
		monitor /*Object*/,
		icon = null /*todo*/,
		callbacksToInsert = new Map() /*Map[str, str]*/,
		initialiseCallback = null /*str|null*/,
		destroyCallback = null /*str|null*/,
		validateCallback = null /*str|null*/,
	) /*str*/ {
		return new Promise((resolve, reject) => {
			let buttonList = [];
			let inputList = [];

			// add canvas and dialog to document
			let canvas = document.getElementById(modalCanvasID);
			if (!canvas) {
				canvas = addElementNode("div", document.body, null, modalCanvasID);
			}
			canvas.style.display = "block";
			let i = 0;
			let dialog = document.getElementById(modalDialogID + i);
			while (dialog) {
				i += 1;
				dialog = document.getElementById(modalDialogID + i);
			}
			dialog = addElementNode("div", document.body, null, modalDialogID + i, modalDialogID);
			dialog.idPrefix = 'dialog' + i;
			dialog.end = function (result) {
				if (destroyCallback) {
					monitor[destroyCallback](dialog);
				}
				resolve(result);
			};
			dialog.load = function (thingsToLoad /*Object*/) {
				for (let elementID in thingsToLoad) {
					let theElement = document.getElementById(dialog.idPrefix + elementID);
					if (theElement == null) {
						continue;
					}
					let elementType = theElement.getAttribute('elementType');
					if ((elementType == 'edit_text') && ((typeof thingsToLoad[elementID]) != 'object')) {
						theElement.value = thingsToLoad[elementID];
					} else if (elementType == 'image') {
						theElement.style.width = thingsToLoad[elementID].width + 'px';
						theElement.style.height = thingsToLoad[elementID].height + 'px';
						theElement.setAttribute('src', 'img/popup_icons/' + thingsToLoad[elementID].name + '.ico');
					} else if (['button', 'static_text', 'link_text'].includes(elementType)) {
						theElement.innerText = thingsToLoad[elementID];
					} else if (
						['popup', 'list_box', 'hier_list_box'].includes(elementType)
						|| ((elementType == 'edit_text') && ((typeof thingsToLoad[elementID]) == 'object'))
					) {
						let currentParent, rootElement;
						if (elementType == 'edit_text') {
							if (theElement.hasAttribute('list')) {
								rootElement = document.getElementById(theElement.getAttribute('list'));
							} else {
								let listName = theElement.id + '_datalist';
								rootElement = addElementNode('datalist', theElement.parentElement, null, listName);
								theElement.setAttribute('list', listName);
								theElement.onclick = function (event) {
									let elWidth = this.style.width.trim();
									if (elWidth.endsWith('px')) {
										if (event.offsetX > Number(this.style.width.replace('px', ''))-16) {
											this.value='';
										}
									} else if (elWidth.endsWith('ch')) {
										if (event.offsetX > Number(this.offsetWidth)-26) {
											this.value='';
										}
									} else {
										throw "Unknown width type for list input:", elWidth
									}
								}
							}
						} else {
							rootElement = theElement;
						}
						for (let i=rootElement.options.length; i--;) {
							currentParent = rootElement.options[i].parentElement;
							currentParent.removeChild(rootElement.options[i]);
							if (
								(currentParent.tagName.toUpperCase() == 'OPTGROUP')
								&& (currentParent.children.length == 0)
							) {
								currentParent.parentElement.removeChild(currentParent);
							}
						}
						for (let optionName in thingsToLoad[elementID]){
							let optionList = {};
							let optionValue = thingsToLoad[elementID][optionName];
							if (Number.isInteger(optionValue) || (typeof optionValue == "boolean")) {
								currentParent = rootElement;
								optionList[optionName] = optionValue;
							} else {
								currentParent = document.createElement('optgroup');
								currentParent.setAttribute('label', optionName);
								rootElement.appendChild(currentParent);
								optionList = optionValue;
							}
							for (let subOptionName in optionList) {
								let option = document.createElement('option');
								option.value = subOptionName;
								option.innerText = subOptionName;
								if (elementType == 'edit_text') {
									if (optionList[subOptionName] > 0) {
										theElement.value = option.value;
									}
								} else {
									option.selected = optionList[subOptionName] > 0;
								}
								currentParent.appendChild(option);
							}
						}
					} else if (['check_box', 'radio'].includes(elementType)) {
						theElement.checked = thingsToLoad[elementID];
					} else if (elementType == 'cluster') {
						theElement.childNodes[0].innerText = thingsToLoad[elementID];
					} else {
						// 'view', 'cluster', 'ok', 'ok_cancel', 'ok_cancel_other', 'gap'
						throw "unknown element type for dialog.load: " + elementType;
					}
				}
			};
			dialog.store = function () /*Object*/ {
				let results = {};
				for (let elementID of inputList) {
					let theElement = document.getElementById(dialog.idPrefix + elementID);
					let elementType = theElement.getAttribute('elementType');
					if (elementType == 'edit_text') {
						results[elementID] = theElement.value;
					} else if (['button', 'static_text', 'link_text'].includes(elementType)) {
						results[elementID] = theElement.innerText;
					} else if (['popup', 'list_box', 'hier_list_box'].includes(elementType)) {
						results[elementID] = {};
						let elementParent, optionValue;
						for (let it = 0; it < theElement.options.length; it++) {
							elementParent = theElement.options[it].parentElement;
							optionValue = ((it == theElement.selectedIndex) ? 1 : -1) * (1 + it);
							if (elementParent.tagName.toUpperCase() == 'OPTGROUP') {
								if (!(elementParent.label in results[elementID])) {
									results[elementID][elementParent.label] = {};
								}
								results[elementID][elementParent.label][theElement.options[it].value] = optionValue;
							} else {
								results[elementID][theElement.options[it].value] = optionValue;
							}
						}
					} else if (['check_box', 'radio'].includes(elementType)) {
						results[elementID] = theElement.checked;
					} else {
						throw "unknown element type for dialog.store: " + elementType;
					}
				}
				return results;
			};
			dialog.setForeColorRed = function (elementId /*str*/) {
				let element = document.getElementById(dialog.idPrefix + elementId);
				if (element) {
					element.style.color = "red";
				}
			};
			dialog.visible = function (thingsToSetVisible /*Object*/) {
				for (let elementID in thingsToSetVisible) {
					if (thingsToSetVisible[elementID]) {
						document.getElementById(dialog.idPrefix + elementID).style.visibility = 'visible';
					} else {
						document.getElementById(dialog.idPrefix + elementID).style.visibility = 'hidden';
					}
				}
			};
			dialog.enable = function (thingsToEnable /*Object*/) {
				for (let elementID in thingsToEnable) {
					let element = document.getElementById(dialog.idPrefix + elementID);
					if (thingsToEnable[elementID]) {
						if (element.style.display == 'none') {
							element.style.display = (
								element.hasAttribute('displayBak') ? element.getAttribute('displayBak') : 'block'
							);
						}
					} else {
						element.setAttribute('displayBak', element.style.display)
						element.style.display = 'none';
					}
				}
			}
			dialog.focus = function (elementID /*String*/) {
				document.getElementById(dialog.idPrefix + elementID).focus();
			}
			dialog._validate = function () {
				if (validateCallback) {
					monitor[validateCallback](dialog);
				}
			}
			dialog.insertEntryInList = function (thingsToInsert /*Object[String, Number]*/) {
				for (let elementID in thingsToInsert) {
					let theElement = document.getElementById(dialog.idPrefix + elementID);
					if (theElement == null) {
						continue;
					}
					let elementType = theElement.getAttribute('elementType');
					if (
						['popup', 'list_box', 'hier_list_box'].includes(elementType)
						|| ((elementType == 'edit_text') && theElement.hasAttribute('list'))
					) {
						let currentParent, rootElement;
						if (elementType == 'edit_text') {
							rootElement = document.getElementById(theElement.getAttribute('list'));
						} else {
							rootElement = theElement;
						}
						for (let optionName in thingsToInsert[elementID]){
							let optionList = {};
							let optionValue = thingsToInsert[elementID][optionName];
							if (Number.isInteger(optionValue) || (typeof optionValue == "boolean")) {
								currentParent = rootElement;
								optionList[optionName] = optionValue;
							} else {
								currentParent = document.createElement('optgroup');
								currentParent.setAttribute('label', optionName);
								rootElement.appendChild(currentParent);
								optionList = optionValue;
							}
							for (let subOptionName in optionList) {
								let option = document.createElement('option');
								option.value = subOptionName;
								option.innerText = subOptionName;
								if (elementType == 'edit_text') {
									if (optionList[subOptionName] > 0) {
										theElement.value = option.value;
									}
								} else {
									option.selected = optionList[subOptionName] > 0;
								}
								currentParent.appendChild(option);
							}
						}
					} else {
						throw "unknown element type for dialog.insertEntryInList: " + elementType;
					}
				}
			}

			dialog.removeAllEntriesFromList = function(listName /*String*/) {
				let theElement = document.getElementById(dialog.idPrefix + listName);
				if (theElement == null) {
					return;
				}
				let elementType = theElement.getAttribute('elementType');
				if (
					['popup', 'list_box', 'hier_list_box'].includes(elementType)
					|| ((elementType == 'edit_text') && theElement.hasAttribute('list'))
				) {
					let currentParent, rootElement;
					if (elementType == 'edit_text') {
						rootElement = document.getElementById(theElement.getAttribute('list'));
					} else {
						rootElement = theElement;
					}
					while (rootElement.lastChild) {
						rootElement.lastChild.remove()
					}
				} else {
					throw "unknown element type for dialog.removeAllEntriesFromList: " + elementType;
				}
			}

			addElementNode("p", dialog, title, dialog.idPrefix + "modalTitle", "modalTitle");
			if (icon) {
				console.log("Warning: icon in dialog not implemented yet");
				// TODO: add icon (addElementNode("img", dialog, "", "modalIcon").src = this.iconNames[icon];)
			}
			let modalContent = addElementNode("div", dialog, "", dialog.idPrefix + "modalContent", "modalContent");

			// add body
			for (let i = 0; i < body.length; i++) {
				this._add_body_recursive(modalContent, body[i], callbacksToInsert, inputList, buttonList, dialog, monitor, resolve);
			}

			// initialisation callback
			if (initialiseCallback !== null) {
				monitor[initialiseCallback](dialog);
			}

			// capture keyboard events
			// TODO: fix this
			// dialog.addEventListener("keydown", (event) => { this._keyboardHandler(event, buttonList); });

		}).finally(this._hide_modal);
	},

	_add_body_recursive: function (parent, body, callbacksToInsert, inputList, buttonList, dialog, monitor, resolve_cb) {
		if (!body.type) {
			throw "no type in body description for execDialog";
		}

		let element;
		let elID = (body.item_id !== undefined) ? dialog.idPrefix + body.item_id : undefined;
		if (body.type == 'view') {
			let style = this._parse_element_style(body);
			element = addElementNode('div', parent, null, elID, 'modalDialogViewElement', style);
			for (let i = 0; i < body.elements.length; i++) {
				this._add_body_recursive(element, body.elements[i], callbacksToInsert, inputList, buttonList, dialog, monitor, resolve_cb);
			}
		} else if (body.type == 'cluster') {
			element = addElementNode('fieldset', parent, null, elID, null, null);
			addElementNode('legend', element, body.name, null, null, null);
			let style = this._parse_element_style(body);
			style.borderWidth = 1;
			let container = addElementNode('div', element, null, null, 'modalDialogViewElement', style);
			for (let i = 0; i < body.elements.length; i++) {
				this._add_body_recursive(container, body.elements[i], callbacksToInsert, inputList, buttonList, dialog, monitor, resolve_cb);
			}
		} else if (body.type == 'static_text') {
			let style = this._parse_element_style(body);
			element = addElementNode('div', parent, body.name, elID, 'modalDialogStaticTextElement', style);
			if (body.item_id) {
				inputList.push(body.item_id);
			}
		} else if (body.type == 'ok') {
			let containerFloat = (body.alignment == 'align_center') ? 'center' : ((body.alignment == 'align_right') ? 'right' : 'left');
			element = addElementNode(
				'div', parent, null, elID, 'modalDialogButtonsElement', {gridTemplateColumns: 'auto', float: containerFloat}
			);
			const { type, item_id, ok_name, other_name, alignment, ...body_style } = body;
			let style = this._parse_element_style(body_style);
			let buttonName = body['ok_name'] || 'OK';
			this._add_button(element, 'ok', buttonName, callbacksToInsert, buttonList, style, dialog, monitor, resolve_cb);
		} else if (body.type == 'ok_cancel') {
			let containerFloat = (body.alignment == 'align_center') ? 'center' : ((body.alignment == 'align_right') ? 'right' : 'left');
			element = addElementNode(
				'div', parent, null, elID, 'modalDialogButtonsElement', {gridTemplateColumns: 'auto auto', float: containerFloat}
			);
			const { type, item_id, ok_name, cancel_name, other_name, alignment, ...body_style } = body;
			let style = this._parse_element_style(body_style);
			for (const buttonID of ['ok', 'cancel']) {
				let buttonName = body[buttonID + '_name'] || buttonID.replace(/^\w/, c => c.toUpperCase());
				this._add_button(element, buttonID, buttonName, callbacksToInsert, buttonList, style, dialog, monitor, resolve_cb);
			}
		} else if (body.type == 'ok_cancel_other') {
			let containerFloat = (body.alignment == 'align_center') ? 'center' : ((body.alignment == 'align_right') ? 'right' : 'left');
			element = addElementNode(
				'div', parent, null, elID, 'modalDialogButtonsElement', {gridTemplateColumns: 'auto auto auto', float: containerFloat}
			);
			const { type, item_id, ok_name, cancel_name, other_name, alignment, ...body_style } = body;
			let style = this._parse_element_style(body_style);
			for (const buttonID of ['ok', 'other', 'cancel']) {
				let buttonName = body[buttonID + '_name'] || buttonID.replace(/^\w/, c => c.toUpperCase());
				this._add_button(element, buttonID, buttonName, callbacksToInsert, buttonList, style, dialog, monitor, resolve_cb);
			}
		} else if (body.type == 'edit_text') {
			let tag = 'input';
			let type = 'text';
			if (body.multiline) {
				tag = 'textarea';
				type = null;
				delete body.multiline;
			}
			let readonly = false;
			if ('readonly' in body) {
				readonly = body.readonly;
				delete body.readonly;
			}
			if (body.password != null) {
				throw "edit_text option password not implemented";
			}
			let listName = null;
			if (body.PopupEdit != null) {
				listName = elID + '_datalist';
				addElementNode('datalist', parent, null, listName);
				delete body.PopupEdit;
			}
			let spinEdit = false;
			if (body.SpinEdit != null) {
				if (!listName) {
					spinEdit = body.SpinEdit;
				}
				delete body.SpinEdit;
			}
			let style = this._parse_element_style(body);
			element = addElementNode(tag, parent, null, elID, null, style);
			if (type != null) {
				element.setAttribute('type', type);
			}
			if (readonly) {
				element.setAttribute('readonly', true);
			}
			if (listName) {
				element.setAttribute('list', listName);
				element.onclick = function (event) {
					let elWidth = this.style.width.trim();
					if (elWidth.endsWith('px')) {
						if (event.offsetX > Number(this.style.width.replace('px', ''))-16) {
							this.value='';
						}
					} else if (elWidth.endsWith('ch')) {
						if (event.offsetX > Number(this.offsetWidth)-26) {
							this.value='';
						}
					} else {
						throw "Unknown width type for list input:", elWidth
					}
				}
			}
			if (spinEdit) {
				element.setAttribute('type', 'number')
				element.setAttribute('step', 1)
			}
			if (callbacksToInsert.has(body.item_id)) {
				element.onchange = async function () {
					await monitor[callbacksToInsert.get(body.item_id)](dialog);
				};
			}
			inputList.push(body.item_id);
		} else if (body.type == 'gap') {
			element = addElementNode('div', parent, null, elID, '', this._parse_element_style(body));
		} else if (body.type == 'image') {
			element = addElementNode('img', parent, null, elID, '', this._parse_element_style(body));
		} else if (body.type == 'popup') {
			let style = this._parse_element_style(body);
			style.height = "27.5px";
			element = addElementNode('select', parent, null, elID, '', style);
			if (callbacksToInsert.has(body.item_id)) {
				element.onchange = async function () {
					await monitor[callbacksToInsert.get(body.item_id)](dialog);
				};
			}
			inputList.push(body.item_id);
		} else if (['list_box', 'hier_list_box'].includes(body.type)) {
			let style = this._parse_element_style(body);
			element = addElementNode('select', parent, null, elID, 'modalDialogListBox', style);
			element.setAttribute('multiple',  true);
			if (callbacksToInsert.has(body.item_id)) {
				element.onclick = async function () {
					await monitor[callbacksToInsert.get(body.item_id)](dialog);
				};
			}
			inputList.push(body.item_id);
		} else if (body.type == 'button') {
			let style = this._parse_element_style(body);
			element = this._add_button(
				parent = parent,
				buttonID = body.item_id,
				buttonName = body.name.replace('&&', '&amp;'),
				callbacksToInsert = callbacksToInsert,
				buttonList = buttonList,
				style = style,
				dialog = dialog,
				monitor = monitor,
				resolve_cb = resolve_cb,
			)
		} else if (body.type == 'link_text') {
			let style = this._parse_element_style(body);
			element = addElementNode('a', parent, null, elID, null, style);
			element.href = "#";
			if (callbacksToInsert.has(body.item_id)) {
				element.onclick = async function () {
					await monitor[callbacksToInsert.get(body.item_id)](dialog);
				};
			}
			inputList.push(body.item_id);
		} else if (body.type == 'radio') {
			let { type, item_id, group_id, name, ...body_style } = body;
			let style = this._parse_element_style(body_style);
			let container = addElementNode('div', parent, null, null, null, style);
			element = addElementNode('input', container, '', elID, null);
			element.setAttribute('type', 'radio');
			element.setAttribute('name', group_id);
			element.setAttribute('value', name);
			let labelElement = addElementNode('label', container, name, null, null);
			labelElement.setAttribute('for', elID);
			inputList.push(body.item_id);
		} else if (body.type == 'check_box') {
			let { type, item_id, name, ...body_style } = body;
			let style = this._parse_element_style(body_style);
			let container = addElementNode('div', parent, null, null, null, style);
			element = addElementNode('input', container, '', elID, null);
			element.setAttribute('type', 'checkbox');
			element.setAttribute('value', name);
			let labelElement = addElementNode('label', container, name, null, null);
			labelElement.setAttribute('for', elID);
			inputList.push(body.item_id);
		} else {
			throw "unimplemented element type for execDialog: " + body.type;
		}
		element.setAttribute('elementType', body.type);
		var oldOnchange = element.onchange;
		element.onchange = function () {
			if (oldOnchange) {
				oldOnchange();
			}
			dialog._validate();
		}
	},

	_keyboardHandler: function (evt, buttonList) {
		evt = evt || window.event;
		let e = {
			target: evt.target || evt.srcElement,
			keyCode: evt.keyCode || evt.which || 0,
			shiftKey: evt.shiftKey,
			ctrlKey: evt.ctrlKey,
			altKey: evt.altKey,
		}
		evt.stopPropagation();
		evt.preventDefault();

		let el = e.target;
		if (el.className != modalDialogButtonLinkClass) {
			return;
		}
		let btn = el.parentNode;

		switch (e.keyCode) {
			case 13:
				if (el.onclick) el.onclick(); // enter
				break;
			case 9:  // tab
				let nextIndex = buttonList.indexOf(btn);
				if (e.shiftKey) {
					nextIndex -= 1;
				} else {
					nextIndex += 1;
				}
				nextIndex = (nextIndex + buttonList.length) % buttonList.length;
				buttonList[nextIndex].focus();
		};
	},

	_hide_modal: function () {
		let i = 0;
		let upperDialog = document.getElementById(modalDialogID + (i + 1));
		while (upperDialog) {
			i += 1;
			upperDialog = document.getElementById(modalDialogID + (i + 1));
		}
		document.body.removeChild(document.getElementById(modalDialogID + i));
		if (i == 0) {
			let canvas = document.getElementById(modalCanvasID);
			if (canvas) {
				canvas.style.display = "none";
			}
		}
	},

	_parse_element_style: function (element /*Object*/) /*Object*/ {
		let styleElements = [
			'item_id', 'type', 'alignment', 'align_children', 'back_color', 'bold', 'char_height', 'char_width', 'font',
			'gradient_direction', 'gradient_type', 'height', 'margin_height', 'name', 'truncate', 'width', 'wrap_name',
			'next_tab',
		];
		let ignoredElements = ['elements', 'skipType'];
		for (let prop in element) {  // TODO: remove this check when complete
			if (!styleElements.includes(prop) && !ignoredElements.includes(prop)) {
				throw "unknown style in element for execDialog: " + prop;
			}
		}
		let style = {};
		if (element.type == 'view') {
			style.display = 'grid';
		}
		if (element.alignment == 'align_top') {
			style.textAlign = 'center';
			style.display = 'grid';
		} else if (element.alignment == 'align_fill') {
			style.textAlign = 'left';
			style.width = '100%';
		} else if (element.alignment == 'align_left') {
			style.textAlign = 'left';
			style.display = 'grid';
		} else if (element.alignment == 'align_right') {
			style.textAlign = 'right';
			style.display = 'grid';
		} else if (element.alignment == 'align_center') {
			style.textAlign = 'center';
			style.display = 'grid';
		} else if (element.alignment == 'align_offscreen') {
			style.display = 'none';
		} else if (element.alignment) {
			throw "unimplemented alignment for element in execDialog: " + element.alignment;
		}
		if (element.font == 'heading') {
			style.fontSize = '18px';
			style.paddingBottom = '5px';
		} else if (element.font == 'dialog') {
			style.fontSize = '14px';
		} else if (element.font == 'title') {
			style.marginTop = '2px';
			style.marginLeft = '2px';
			style.marginBottom = '2px';
			style.marginRight = '2px';
			style.fontSize = '20px';
			style.fontWeight = 'bold';
		} else if (element.font == 'palette') {
			style.fontSize = '12px';
		} else if (element.font == undefined) {
			style.fontSize = '14px';
		} else {
			throw "unimplemented font type for static_text element in execDialog: " + element.font;
		}
		if (element.char_width) {
			let width = element.char_width * 21 / Number(style.fontSize.slice(0, 2));
			if (element.type == 'edit_text') {
				width -= 1.03; // subtract padding
			}
			style.width = String(width) + 'ch';
		}
		if (element.width) {
			let width = element.width * 4 / 3;
			if (element.type == 'edit_text') {
				width -= 6; // subtract padding
			}
			style.width = String(width) + 'px';
		}
		// } else {
		// 	if (element.name) {
		// 		style.width = String(element.name.length + 2) + 'ch';
		// 	}
		// }
		if (element.bold) {
			style.fontWeight = 'bold';
		}
		if (element.char_height) {
			let height = element.char_height * 20 / Number(style.fontSize.slice(0, 2));
			if (element.type == 'edit_text') {
				height -= 1.03; // subtract padding
			}
			style.height = String(height) + 'ch';
		}
		if (element.height) {
			let height = element.height * 4 / 3;
			if (element.type == 'edit_text') {
				height -= 6; // subtract padding
			}
			style.height = String(height) + 'px';
		}
		if (element.wrap_name) {
			style.overflowWrap = 'break-word';
		}
		if (element.align_children == 'align_top') {
			style.display = 'flex';
			style.flexDirection = 'row';
			style.alignItems = 'center';
			style.justifyContent = 'center';
		} else if (element.align_children == 'align_left') {
			style.display = 'flex';
			style.flexDirection = 'column';
			style.alignItems = 'left';
			style.justifyContent = 'left';
		} else if (element.align_children == 'align_right') {
			style.display = 'flex';
			style.flexDirection = 'column';
			style.alignItems = 'right';
			style.justifyContent = 'right';
		} else if (element.align_children == 'align_row') {
			style.display = 'flex';
			style.flexDirection = 'row';
		} else if (element.align_children == 'align_center') {
			style.display = 'flex';
			style.flexDirection = 'column';
			style.alignItems = 'center';
			style.justifyContent = 'center';
		} else if (element.align_children == 'align_distribute') {
			style.display = 'inline-flex';
			style.width = '100%';
		} else if (element.align_children) {
			throw "align_children '" + element.align_children + "' not yet implemented";
		}
		if (element.truncate == 'truncate_end') {
			style.textOverflow = 'ellipsis';
		} else if (element.truncate) {
			throw "truncate '" + element.truncate + "' not yet implemented";
		}
		if (element.margin_height) {
			style.marginTop = element.margin_height;
		}
		if (element.back_color == 'dialogBackground') {
			style.backgroundColor = 'white';
		} else if (element.back_color == 'windowBackground') {
			style.backgroundColor = 'var(--background)';
		} else if (element.back_color) {
			throw "back_color '" + element.back_color + "' not yet implemented";
		}
		if (element.gradient_direction == 'topToBottom') {
			// TODO
		} else if (element.gradient_direction) {
			throw "gradient_direction '" + element.gradient_direction + "' not yet implemented";
		}
		if (element.gradient_type == 'normalToDark') {
			// TODO: actual gradient
			style.backgroundColor = '#d0d0d0';
		} else if (element.gradient_type) {
			throw "gradient_type '" + element.gradient_type + "' not yet implemented";
		}
		return style;
	},

	_add_button: function (parent, buttonID, buttonName, callbacksToInsert, buttonList, style, dialog, monitor, resolve_cb) {
		let button = addElementNode("p", parent, null, null, "modalDialogButton", style);
		let link = addElementNode("a", button, buttonName, dialog.idPrefix + buttonID, modalDialogButtonLinkClass);
		link.href = "#";
		link.setAttribute('tabindex', "0");
		if (callbacksToInsert.has(buttonID)) {
			link.onclick = async function () {
				await monitor[callbacksToInsert.get(buttonID)](dialog);
				if (['commit', 'ok'].includes(callbacksToInsert.get(buttonID))) {
					dialog.end(buttonID);
				}
			};
		} else {
			link.onclick = function () {
				dialog.end(buttonID);
			};
		}
		if (buttonList.length == 0) {
			link.focus();
		}
		buttonList.push(button);
		return link;
	},
};
