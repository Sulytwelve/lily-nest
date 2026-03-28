function t(t,e,i,o){var s,r=arguments.length,a=r<3?e:null===o?o=Object.getOwnPropertyDescriptor(e,i):o;if("object"==typeof Reflect&&"function"==typeof Reflect.decorate)a=Reflect.decorate(t,e,i,o);else for(var n=t.length-1;n>=0;n--)(s=t[n])&&(a=(r<3?s(a):r>3?s(e,i,a):s(e,i))||a);return r>3&&a&&Object.defineProperty(e,i,a),a}"function"==typeof SuppressedError&&SuppressedError;
/**
 * @license
 * Copyright 2017 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */
const e=t=>(e,i)=>{void 0!==i?i.addInitializer(()=>{customElements.define(t,e)}):customElements.define(t,e)},i=globalThis,o=i.ShadowRoot&&(void 0===i.ShadyCSS||i.ShadyCSS.nativeShadow)&&"adoptedStyleSheets"in Document.prototype&&"replace"in CSSStyleSheet.prototype,s=Symbol(),r=new WeakMap;
/**
 * @license
 * Copyright 2019 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */let a=class{constructor(t,e,i){if(this._$cssResult$=!0,i!==s)throw Error("CSSResult is not constructable. Use `unsafeCSS` or `css` instead.");this.cssText=t,this.t=e}get styleSheet(){let t=this.o;const e=this.t;if(o&&void 0===t){const i=void 0!==e&&1===e.length;i&&(t=r.get(e)),void 0===t&&((this.o=t=new CSSStyleSheet).replaceSync(this.cssText),i&&r.set(e,t))}return t}toString(){return this.cssText}};const n=(t,...e)=>{const i=1===t.length?t[0]:e.reduce((e,i,o)=>e+(t=>{if(!0===t._$cssResult$)return t.cssText;if("number"==typeof t)return t;throw Error("Value passed to 'css' function must be a 'css' function result: "+t+". Use 'unsafeCSS' to pass non-literal values, but take care to ensure page security.")})(i)+t[o+1],t[0]);return new a(i,t,s)},l=o?t=>t:t=>t instanceof CSSStyleSheet?(t=>{let e="";for(const i of t.cssRules)e+=i.cssText;return(t=>new a("string"==typeof t?t:t+"",void 0,s))(e)})(t):t,{is:d,defineProperty:c,getOwnPropertyDescriptor:h,getOwnPropertyNames:p,getOwnPropertySymbols:u,getPrototypeOf:m}=Object,v=globalThis,f=v.trustedTypes,b=f?f.emptyScript:"",y=v.reactiveElementPolyfillSupport,g=(t,e)=>t,x={toAttribute(t,e){switch(e){case Boolean:t=t?b:null;break;case Object:case Array:t=null==t?t:JSON.stringify(t)}return t},fromAttribute(t,e){let i=t;switch(e){case Boolean:i=null!==t;break;case Number:i=null===t?null:Number(t);break;case Object:case Array:try{i=JSON.parse(t)}catch(t){i=null}}return i}},_=(t,e)=>!d(t,e),w={attribute:!0,type:String,converter:x,reflect:!1,useDefault:!1,hasChanged:_};
/**
 * @license
 * Copyright 2017 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */Symbol.metadata??=Symbol("metadata"),v.litPropertyMetadata??=new WeakMap;let $=class extends HTMLElement{static addInitializer(t){this._$Ei(),(this.l??=[]).push(t)}static get observedAttributes(){return this.finalize(),this._$Eh&&[...this._$Eh.keys()]}static createProperty(t,e=w){if(e.state&&(e.attribute=!1),this._$Ei(),this.prototype.hasOwnProperty(t)&&((e=Object.create(e)).wrapped=!0),this.elementProperties.set(t,e),!e.noAccessor){const i=Symbol(),o=this.getPropertyDescriptor(t,i,e);void 0!==o&&c(this.prototype,t,o)}}static getPropertyDescriptor(t,e,i){const{get:o,set:s}=h(this.prototype,t)??{get(){return this[e]},set(t){this[e]=t}};return{get:o,set(e){const r=o?.call(this);s?.call(this,e),this.requestUpdate(t,r,i)},configurable:!0,enumerable:!0}}static getPropertyOptions(t){return this.elementProperties.get(t)??w}static _$Ei(){if(this.hasOwnProperty(g("elementProperties")))return;const t=m(this);t.finalize(),void 0!==t.l&&(this.l=[...t.l]),this.elementProperties=new Map(t.elementProperties)}static finalize(){if(this.hasOwnProperty(g("finalized")))return;if(this.finalized=!0,this._$Ei(),this.hasOwnProperty(g("properties"))){const t=this.properties,e=[...p(t),...u(t)];for(const i of e)this.createProperty(i,t[i])}const t=this[Symbol.metadata];if(null!==t){const e=litPropertyMetadata.get(t);if(void 0!==e)for(const[t,i]of e)this.elementProperties.set(t,i)}this._$Eh=new Map;for(const[t,e]of this.elementProperties){const i=this._$Eu(t,e);void 0!==i&&this._$Eh.set(i,t)}this.elementStyles=this.finalizeStyles(this.styles)}static finalizeStyles(t){const e=[];if(Array.isArray(t)){const i=new Set(t.flat(1/0).reverse());for(const t of i)e.unshift(l(t))}else void 0!==t&&e.push(l(t));return e}static _$Eu(t,e){const i=e.attribute;return!1===i?void 0:"string"==typeof i?i:"string"==typeof t?t.toLowerCase():void 0}constructor(){super(),this._$Ep=void 0,this.isUpdatePending=!1,this.hasUpdated=!1,this._$Em=null,this._$Ev()}_$Ev(){this._$ES=new Promise(t=>this.enableUpdating=t),this._$AL=new Map,this._$E_(),this.requestUpdate(),this.constructor.l?.forEach(t=>t(this))}addController(t){(this._$EO??=new Set).add(t),void 0!==this.renderRoot&&this.isConnected&&t.hostConnected?.()}removeController(t){this._$EO?.delete(t)}_$E_(){const t=new Map,e=this.constructor.elementProperties;for(const i of e.keys())this.hasOwnProperty(i)&&(t.set(i,this[i]),delete this[i]);t.size>0&&(this._$Ep=t)}createRenderRoot(){const t=this.shadowRoot??this.attachShadow(this.constructor.shadowRootOptions);return((t,e)=>{if(o)t.adoptedStyleSheets=e.map(t=>t instanceof CSSStyleSheet?t:t.styleSheet);else for(const o of e){const e=document.createElement("style"),s=i.litNonce;void 0!==s&&e.setAttribute("nonce",s),e.textContent=o.cssText,t.appendChild(e)}})(t,this.constructor.elementStyles),t}connectedCallback(){this.renderRoot??=this.createRenderRoot(),this.enableUpdating(!0),this._$EO?.forEach(t=>t.hostConnected?.())}enableUpdating(t){}disconnectedCallback(){this._$EO?.forEach(t=>t.hostDisconnected?.())}attributeChangedCallback(t,e,i){this._$AK(t,i)}_$ET(t,e){const i=this.constructor.elementProperties.get(t),o=this.constructor._$Eu(t,i);if(void 0!==o&&!0===i.reflect){const s=(void 0!==i.converter?.toAttribute?i.converter:x).toAttribute(e,i.type);this._$Em=t,null==s?this.removeAttribute(o):this.setAttribute(o,s),this._$Em=null}}_$AK(t,e){const i=this.constructor,o=i._$Eh.get(t);if(void 0!==o&&this._$Em!==o){const t=i.getPropertyOptions(o),s="function"==typeof t.converter?{fromAttribute:t.converter}:void 0!==t.converter?.fromAttribute?t.converter:x;this._$Em=o;const r=s.fromAttribute(e,t.type);this[o]=r??this._$Ej?.get(o)??r,this._$Em=null}}requestUpdate(t,e,i,o=!1,s){if(void 0!==t){const r=this.constructor;if(!1===o&&(s=this[t]),i??=r.getPropertyOptions(t),!((i.hasChanged??_)(s,e)||i.useDefault&&i.reflect&&s===this._$Ej?.get(t)&&!this.hasAttribute(r._$Eu(t,i))))return;this.C(t,e,i)}!1===this.isUpdatePending&&(this._$ES=this._$EP())}C(t,e,{useDefault:i,reflect:o,wrapped:s},r){i&&!(this._$Ej??=new Map).has(t)&&(this._$Ej.set(t,r??e??this[t]),!0!==s||void 0!==r)||(this._$AL.has(t)||(this.hasUpdated||i||(e=void 0),this._$AL.set(t,e)),!0===o&&this._$Em!==t&&(this._$Eq??=new Set).add(t))}async _$EP(){this.isUpdatePending=!0;try{await this._$ES}catch(t){Promise.reject(t)}const t=this.scheduleUpdate();return null!=t&&await t,!this.isUpdatePending}scheduleUpdate(){return this.performUpdate()}performUpdate(){if(!this.isUpdatePending)return;if(!this.hasUpdated){if(this.renderRoot??=this.createRenderRoot(),this._$Ep){for(const[t,e]of this._$Ep)this[t]=e;this._$Ep=void 0}const t=this.constructor.elementProperties;if(t.size>0)for(const[e,i]of t){const{wrapped:t}=i,o=this[e];!0!==t||this._$AL.has(e)||void 0===o||this.C(e,void 0,i,o)}}let t=!1;const e=this._$AL;try{t=this.shouldUpdate(e),t?(this.willUpdate(e),this._$EO?.forEach(t=>t.hostUpdate?.()),this.update(e)):this._$EM()}catch(e){throw t=!1,this._$EM(),e}t&&this._$AE(e)}willUpdate(t){}_$AE(t){this._$EO?.forEach(t=>t.hostUpdated?.()),this.hasUpdated||(this.hasUpdated=!0,this.firstUpdated(t)),this.updated(t)}_$EM(){this._$AL=new Map,this.isUpdatePending=!1}get updateComplete(){return this.getUpdateComplete()}getUpdateComplete(){return this._$ES}shouldUpdate(t){return!0}update(t){this._$Eq&&=this._$Eq.forEach(t=>this._$ET(t,this[t])),this._$EM()}updated(t){}firstUpdated(t){}};$.elementStyles=[],$.shadowRootOptions={mode:"open"},$[g("elementProperties")]=new Map,$[g("finalized")]=new Map,y?.({ReactiveElement:$}),(v.reactiveElementVersions??=[]).push("2.1.2");
/**
 * @license
 * Copyright 2017 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */
const C={attribute:!0,type:String,converter:x,reflect:!1,hasChanged:_},E=(t=C,e,i)=>{const{kind:o,metadata:s}=i;let r=globalThis.litPropertyMetadata.get(s);if(void 0===r&&globalThis.litPropertyMetadata.set(s,r=new Map),"setter"===o&&((t=Object.create(t)).wrapped=!0),r.set(i.name,t),"accessor"===o){const{name:o}=i;return{set(i){const s=e.get.call(this);e.set.call(this,i),this.requestUpdate(o,s,t,!0,i)},init(e){return void 0!==e&&this.C(o,void 0,t,e),e}}}if("setter"===o){const{name:o}=i;return function(i){const s=this[o];e.call(this,i),this.requestUpdate(o,s,t,!0,i)}}throw Error("Unsupported decorator location: "+o)};function A(t){return(e,i)=>"object"==typeof i?E(t,e,i):((t,e,i)=>{const o=e.hasOwnProperty(i);return e.constructor.createProperty(i,t),o?Object.getOwnPropertyDescriptor(e,i):void 0})(t,e,i)}
/**
 * @license
 * Copyright 2017 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */function k(t){return A({...t,state:!0,attribute:!1})}
/**
 * @license
 * Copyright 2017 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */const I=(t,e,i)=>(i.configurable=!0,i.enumerable=!0,Reflect.decorate&&"object"!=typeof e&&Object.defineProperty(t,e,i),i);
/**
 * @license
 * Copyright 2017 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */function S(t,e){return(e,i,o)=>I(e,i,{get(){return(e=>e.renderRoot?.querySelector(t)??null)(this)}})}
/**
 * @license
 * Copyright 2017 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */let R;
/**
 * @license
 * Copyright 2021 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */
function P(t){return(e,i)=>{const{slot:o,selector:s}=t??{},r="slot"+(o?`[name=${o}]`:":not([name])");return I(e,i,{get(){const e=this.renderRoot?.querySelector(r),i=e?.assignedElements(t)??[];return void 0===s?i:i.filter(t=>t.matches(s))}})}}
/**
 * @license
 * Copyright 2017 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */
/**
 * @license
 * Copyright 2017 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */
const O=globalThis,T=t=>t,z=O.trustedTypes,L=z?z.createPolicy("lit-html",{createHTML:t=>t}):void 0,N="$lit$",D=`lit$${Math.random().toFixed(9).slice(2)}$`,B="?"+D,U=`<${B}>`,H=document,F=()=>H.createComment(""),M=t=>null===t||"object"!=typeof t&&"function"!=typeof t,W=Array.isArray,q="[ \t\n\f\r]",j=/<(?:(!--|\/[^a-zA-Z])|(\/?[a-zA-Z][^>\s]*)|(\/?$))/g,V=/-->/g,K=/>/g,G=RegExp(`>|${q}(?:([^\\s"'>=/]+)(${q}*=${q}*(?:[^ \t\n\f\r"'\`<>=]|("|')|))|$)`,"g"),Y=/'/g,X=/"/g,Z=/^(?:script|style|textarea|title)$/i,J=(t=>(e,...i)=>({_$litType$:t,strings:e,values:i}))(1),Q=Symbol.for("lit-noChange"),tt=Symbol.for("lit-nothing"),et=new WeakMap,it=H.createTreeWalker(H,129);function ot(t,e){if(!W(t)||!t.hasOwnProperty("raw"))throw Error("invalid template strings array");return void 0!==L?L.createHTML(e):e}const st=(t,e)=>{const i=t.length-1,o=[];let s,r=2===e?"<svg>":3===e?"<math>":"",a=j;for(let e=0;e<i;e++){const i=t[e];let n,l,d=-1,c=0;for(;c<i.length&&(a.lastIndex=c,l=a.exec(i),null!==l);)c=a.lastIndex,a===j?"!--"===l[1]?a=V:void 0!==l[1]?a=K:void 0!==l[2]?(Z.test(l[2])&&(s=RegExp("</"+l[2],"g")),a=G):void 0!==l[3]&&(a=G):a===G?">"===l[0]?(a=s??j,d=-1):void 0===l[1]?d=-2:(d=a.lastIndex-l[2].length,n=l[1],a=void 0===l[3]?G:'"'===l[3]?X:Y):a===X||a===Y?a=G:a===V||a===K?a=j:(a=G,s=void 0);const h=a===G&&t[e+1].startsWith("/>")?" ":"";r+=a===j?i+U:d>=0?(o.push(n),i.slice(0,d)+N+i.slice(d)+D+h):i+D+(-2===d?e:h)}return[ot(t,r+(t[i]||"<?>")+(2===e?"</svg>":3===e?"</math>":"")),o]};class rt{constructor({strings:t,_$litType$:e},i){let o;this.parts=[];let s=0,r=0;const a=t.length-1,n=this.parts,[l,d]=st(t,e);if(this.el=rt.createElement(l,i),it.currentNode=this.el.content,2===e||3===e){const t=this.el.content.firstChild;t.replaceWith(...t.childNodes)}for(;null!==(o=it.nextNode())&&n.length<a;){if(1===o.nodeType){if(o.hasAttributes())for(const t of o.getAttributeNames())if(t.endsWith(N)){const e=d[r++],i=o.getAttribute(t).split(D),a=/([.?@])?(.*)/.exec(e);n.push({type:1,index:s,name:a[2],strings:i,ctor:"."===a[1]?ct:"?"===a[1]?ht:"@"===a[1]?pt:dt}),o.removeAttribute(t)}else t.startsWith(D)&&(n.push({type:6,index:s}),o.removeAttribute(t));if(Z.test(o.tagName)){const t=o.textContent.split(D),e=t.length-1;if(e>0){o.textContent=z?z.emptyScript:"";for(let i=0;i<e;i++)o.append(t[i],F()),it.nextNode(),n.push({type:2,index:++s});o.append(t[e],F())}}}else if(8===o.nodeType)if(o.data===B)n.push({type:2,index:s});else{let t=-1;for(;-1!==(t=o.data.indexOf(D,t+1));)n.push({type:7,index:s}),t+=D.length-1}s++}}static createElement(t,e){const i=H.createElement("template");return i.innerHTML=t,i}}function at(t,e,i=t,o){if(e===Q)return e;let s=void 0!==o?i._$Co?.[o]:i._$Cl;const r=M(e)?void 0:e._$litDirective$;return s?.constructor!==r&&(s?._$AO?.(!1),void 0===r?s=void 0:(s=new r(t),s._$AT(t,i,o)),void 0!==o?(i._$Co??=[])[o]=s:i._$Cl=s),void 0!==s&&(e=at(t,s._$AS(t,e.values),s,o)),e}class nt{constructor(t,e){this._$AV=[],this._$AN=void 0,this._$AD=t,this._$AM=e}get parentNode(){return this._$AM.parentNode}get _$AU(){return this._$AM._$AU}u(t){const{el:{content:e},parts:i}=this._$AD,o=(t?.creationScope??H).importNode(e,!0);it.currentNode=o;let s=it.nextNode(),r=0,a=0,n=i[0];for(;void 0!==n;){if(r===n.index){let e;2===n.type?e=new lt(s,s.nextSibling,this,t):1===n.type?e=new n.ctor(s,n.name,n.strings,this,t):6===n.type&&(e=new ut(s,this,t)),this._$AV.push(e),n=i[++a]}r!==n?.index&&(s=it.nextNode(),r++)}return it.currentNode=H,o}p(t){let e=0;for(const i of this._$AV)void 0!==i&&(void 0!==i.strings?(i._$AI(t,i,e),e+=i.strings.length-2):i._$AI(t[e])),e++}}class lt{get _$AU(){return this._$AM?._$AU??this._$Cv}constructor(t,e,i,o){this.type=2,this._$AH=tt,this._$AN=void 0,this._$AA=t,this._$AB=e,this._$AM=i,this.options=o,this._$Cv=o?.isConnected??!0}get parentNode(){let t=this._$AA.parentNode;const e=this._$AM;return void 0!==e&&11===t?.nodeType&&(t=e.parentNode),t}get startNode(){return this._$AA}get endNode(){return this._$AB}_$AI(t,e=this){t=at(this,t,e),M(t)?t===tt||null==t||""===t?(this._$AH!==tt&&this._$AR(),this._$AH=tt):t!==this._$AH&&t!==Q&&this._(t):void 0!==t._$litType$?this.$(t):void 0!==t.nodeType?this.T(t):(t=>W(t)||"function"==typeof t?.[Symbol.iterator])(t)?this.k(t):this._(t)}O(t){return this._$AA.parentNode.insertBefore(t,this._$AB)}T(t){this._$AH!==t&&(this._$AR(),this._$AH=this.O(t))}_(t){this._$AH!==tt&&M(this._$AH)?this._$AA.nextSibling.data=t:this.T(H.createTextNode(t)),this._$AH=t}$(t){const{values:e,_$litType$:i}=t,o="number"==typeof i?this._$AC(t):(void 0===i.el&&(i.el=rt.createElement(ot(i.h,i.h[0]),this.options)),i);if(this._$AH?._$AD===o)this._$AH.p(e);else{const t=new nt(o,this),i=t.u(this.options);t.p(e),this.T(i),this._$AH=t}}_$AC(t){let e=et.get(t.strings);return void 0===e&&et.set(t.strings,e=new rt(t)),e}k(t){W(this._$AH)||(this._$AH=[],this._$AR());const e=this._$AH;let i,o=0;for(const s of t)o===e.length?e.push(i=new lt(this.O(F()),this.O(F()),this,this.options)):i=e[o],i._$AI(s),o++;o<e.length&&(this._$AR(i&&i._$AB.nextSibling,o),e.length=o)}_$AR(t=this._$AA.nextSibling,e){for(this._$AP?.(!1,!0,e);t!==this._$AB;){const e=T(t).nextSibling;T(t).remove(),t=e}}setConnected(t){void 0===this._$AM&&(this._$Cv=t,this._$AP?.(t))}}class dt{get tagName(){return this.element.tagName}get _$AU(){return this._$AM._$AU}constructor(t,e,i,o,s){this.type=1,this._$AH=tt,this._$AN=void 0,this.element=t,this.name=e,this._$AM=o,this.options=s,i.length>2||""!==i[0]||""!==i[1]?(this._$AH=Array(i.length-1).fill(new String),this.strings=i):this._$AH=tt}_$AI(t,e=this,i,o){const s=this.strings;let r=!1;if(void 0===s)t=at(this,t,e,0),r=!M(t)||t!==this._$AH&&t!==Q,r&&(this._$AH=t);else{const o=t;let a,n;for(t=s[0],a=0;a<s.length-1;a++)n=at(this,o[i+a],e,a),n===Q&&(n=this._$AH[a]),r||=!M(n)||n!==this._$AH[a],n===tt?t=tt:t!==tt&&(t+=(n??"")+s[a+1]),this._$AH[a]=n}r&&!o&&this.j(t)}j(t){t===tt?this.element.removeAttribute(this.name):this.element.setAttribute(this.name,t??"")}}class ct extends dt{constructor(){super(...arguments),this.type=3}j(t){this.element[this.name]=t===tt?void 0:t}}class ht extends dt{constructor(){super(...arguments),this.type=4}j(t){this.element.toggleAttribute(this.name,!!t&&t!==tt)}}class pt extends dt{constructor(t,e,i,o,s){super(t,e,i,o,s),this.type=5}_$AI(t,e=this){if((t=at(this,t,e,0)??tt)===Q)return;const i=this._$AH,o=t===tt&&i!==tt||t.capture!==i.capture||t.once!==i.once||t.passive!==i.passive,s=t!==tt&&(i===tt||o);o&&this.element.removeEventListener(this.name,this,i),s&&this.element.addEventListener(this.name,this,t),this._$AH=t}handleEvent(t){"function"==typeof this._$AH?this._$AH.call(this.options?.host??this.element,t):this._$AH.handleEvent(t)}}class ut{constructor(t,e,i){this.element=t,this.type=6,this._$AN=void 0,this._$AM=e,this.options=i}get _$AU(){return this._$AM._$AU}_$AI(t){at(this,t)}}const mt=O.litHtmlPolyfillSupport;mt?.(rt,lt),(O.litHtmlVersions??=[]).push("3.3.2");const vt=globalThis;
/**
 * @license
 * Copyright 2017 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */let ft=class extends ${constructor(){super(...arguments),this.renderOptions={host:this},this._$Do=void 0}createRenderRoot(){const t=super.createRenderRoot();return this.renderOptions.renderBefore??=t.firstChild,t}update(t){const e=this.render();this.hasUpdated||(this.renderOptions.isConnected=this.isConnected),super.update(t),this._$Do=((t,e,i)=>{const o=i?.renderBefore??e;let s=o._$litPart$;if(void 0===s){const t=i?.renderBefore??null;o._$litPart$=s=new lt(e.insertBefore(F(),t),t,void 0,i??{})}return s._$AI(t),s})(e,this.renderRoot,this.renderOptions)}connectedCallback(){super.connectedCallback(),this._$Do?.setConnected(!0)}disconnectedCallback(){super.disconnectedCallback(),this._$Do?.setConnected(!1)}render(){return Q}};ft._$litElement$=!0,ft.finalized=!0,vt.litElementHydrateSupport?.({LitElement:ft});const bt=vt.litElementPolyfillSupport;bt?.({LitElement:ft}),(vt.litElementVersions??=[]).push("4.2.2");
/**
 * @license
 * Copyright 2022 Google LLC
 * SPDX-License-Identifier: Apache-2.0
 */
class yt extends ft{render(){return J`<slot></slot>`}connectedCallback(){super.connectedCallback();"false"!==this.getAttribute("aria-hidden")?this.setAttribute("aria-hidden","true"):this.removeAttribute("aria-hidden")}}
/**
 * @license
 * Copyright 2024 Google LLC
 * SPDX-License-Identifier: Apache-2.0
 */const gt=n`:host{font-size:var(--md-icon-size, 24px);width:var(--md-icon-size, 24px);height:var(--md-icon-size, 24px);color:inherit;font-variation-settings:inherit;font-weight:400;font-family:var(--md-icon-font, Material Symbols Outlined);display:inline-flex;font-style:normal;place-items:center;place-content:center;line-height:1;overflow:hidden;letter-spacing:normal;text-transform:none;user-select:none;white-space:nowrap;word-wrap:normal;flex-shrink:0;-webkit-font-smoothing:antialiased;text-rendering:optimizeLegibility;-moz-osx-font-smoothing:grayscale}::slotted(svg){fill:currentColor}::slotted(*){height:100%;width:100%}
`
/**
 * @license
 * Copyright 2022 Google LLC
 * SPDX-License-Identifier: Apache-2.0
 */;let xt=class extends yt{};xt.styles=[gt],xt=t([e("md-icon")],xt);
/**
 * @license
 * Copyright 2023 Google LLC
 * SPDX-License-Identifier: Apache-2.0
 */
const _t=Symbol("attachableController");let wt;wt=new MutationObserver(t=>{for(const e of t)e.target[_t]?.hostConnected()});class $t{get htmlFor(){return this.host.getAttribute("for")}set htmlFor(t){null===t?this.host.removeAttribute("for"):this.host.setAttribute("for",t)}get control(){return this.host.hasAttribute("for")?this.htmlFor&&this.host.isConnected?this.host.getRootNode().querySelector(`#${this.htmlFor}`):null:this.currentControl||this.host.parentElement}set control(t){t?this.attach(t):this.detach()}constructor(t,e){this.host=t,this.onControlChange=e,this.currentControl=null,t.addController(this),t[_t]=this,wt?.observe(t,{attributeFilter:["for"]})}attach(t){t!==this.currentControl&&(this.setCurrentControl(t),this.host.removeAttribute("for"))}detach(){this.setCurrentControl(null),this.host.setAttribute("for","")}hostConnected(){this.setCurrentControl(this.control)}hostDisconnected(){this.setCurrentControl(null)}setCurrentControl(t){this.onControlChange(this.currentControl,t),this.currentControl=t}}
/**
 * @license
 * Copyright 2021 Google LLC
 * SPDX-License-Identifier: Apache-2.0
 */const Ct=["focusin","focusout","pointerdown"];class Et extends ft{constructor(){super(...arguments),this.visible=!1,this.inward=!1,this.attachableController=new $t(this,this.onControlChange.bind(this))}get htmlFor(){return this.attachableController.htmlFor}set htmlFor(t){this.attachableController.htmlFor=t}get control(){return this.attachableController.control}set control(t){this.attachableController.control=t}attach(t){this.attachableController.attach(t)}detach(){this.attachableController.detach()}connectedCallback(){super.connectedCallback(),this.setAttribute("aria-hidden","true")}handleEvent(t){if(!t[At]){switch(t.type){default:return;case"focusin":this.visible=this.control?.matches(":focus-visible")??!1;break;case"focusout":case"pointerdown":this.visible=!1}t[At]=!0}}onControlChange(t,e){for(const i of Ct)t?.removeEventListener(i,this),e?.addEventListener(i,this)}update(t){t.has("visible")&&this.dispatchEvent(new Event("visibility-changed")),super.update(t)}}t([A({type:Boolean,reflect:!0})],Et.prototype,"visible",void 0),t([A({type:Boolean,reflect:!0})],Et.prototype,"inward",void 0);const At=Symbol("handledByFocusRing"),kt=n`:host{animation-delay:0s,calc(var(--md-focus-ring-duration, 600ms)*.25);animation-duration:calc(var(--md-focus-ring-duration, 600ms)*.25),calc(var(--md-focus-ring-duration, 600ms)*.75);animation-timing-function:cubic-bezier(0.2, 0, 0, 1);box-sizing:border-box;color:var(--md-focus-ring-color, var(--md-sys-color-secondary, #625b71));display:none;pointer-events:none;position:absolute}:host([visible]){display:flex}:host(:not([inward])){animation-name:outward-grow,outward-shrink;border-end-end-radius:calc(var(--md-focus-ring-shape-end-end, var(--md-focus-ring-shape, var(--md-sys-shape-corner-full, 9999px))) + var(--md-focus-ring-outward-offset, 2px));border-end-start-radius:calc(var(--md-focus-ring-shape-end-start, var(--md-focus-ring-shape, var(--md-sys-shape-corner-full, 9999px))) + var(--md-focus-ring-outward-offset, 2px));border-start-end-radius:calc(var(--md-focus-ring-shape-start-end, var(--md-focus-ring-shape, var(--md-sys-shape-corner-full, 9999px))) + var(--md-focus-ring-outward-offset, 2px));border-start-start-radius:calc(var(--md-focus-ring-shape-start-start, var(--md-focus-ring-shape, var(--md-sys-shape-corner-full, 9999px))) + var(--md-focus-ring-outward-offset, 2px));inset:calc(-1*var(--md-focus-ring-outward-offset, 2px));outline:var(--md-focus-ring-width, 3px) solid currentColor}:host([inward]){animation-name:inward-grow,inward-shrink;border-end-end-radius:calc(var(--md-focus-ring-shape-end-end, var(--md-focus-ring-shape, var(--md-sys-shape-corner-full, 9999px))) - var(--md-focus-ring-inward-offset, 0px));border-end-start-radius:calc(var(--md-focus-ring-shape-end-start, var(--md-focus-ring-shape, var(--md-sys-shape-corner-full, 9999px))) - var(--md-focus-ring-inward-offset, 0px));border-start-end-radius:calc(var(--md-focus-ring-shape-start-end, var(--md-focus-ring-shape, var(--md-sys-shape-corner-full, 9999px))) - var(--md-focus-ring-inward-offset, 0px));border-start-start-radius:calc(var(--md-focus-ring-shape-start-start, var(--md-focus-ring-shape, var(--md-sys-shape-corner-full, 9999px))) - var(--md-focus-ring-inward-offset, 0px));border:var(--md-focus-ring-width, 3px) solid currentColor;inset:var(--md-focus-ring-inward-offset, 0px)}@keyframes outward-grow{from{outline-width:0}to{outline-width:var(--md-focus-ring-active-width, 8px)}}@keyframes outward-shrink{from{outline-width:var(--md-focus-ring-active-width, 8px)}}@keyframes inward-grow{from{border-width:0}to{border-width:var(--md-focus-ring-active-width, 8px)}}@keyframes inward-shrink{from{border-width:var(--md-focus-ring-active-width, 8px)}}@media(prefers-reduced-motion){:host{animation:none}}
`
/**
 * @license
 * Copyright 2021 Google LLC
 * SPDX-License-Identifier: Apache-2.0
 */;
/**
 * @license
 * Copyright 2024 Google LLC
 * SPDX-License-Identifier: Apache-2.0
 */let It=class extends Et{};It.styles=[kt],It=t([e("md-focus-ring")],It);
/**
 * @license
 * Copyright 2017 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */
const St=1,Rt=t=>(...e)=>({_$litDirective$:t,values:e});let Pt=class{constructor(t){}get _$AU(){return this._$AM._$AU}_$AT(t,e,i){this._$Ct=t,this._$AM=e,this._$Ci=i}_$AS(t,e){return this.update(t,e)}update(t,e){return this.render(...e)}};
/**
 * @license
 * Copyright 2018 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */const Ot=Rt(class extends Pt{constructor(t){if(super(t),t.type!==St||"class"!==t.name||t.strings?.length>2)throw Error("`classMap()` can only be used in the `class` attribute and must be the only part in the attribute.")}render(t){return" "+Object.keys(t).filter(e=>t[e]).join(" ")+" "}update(t,[e]){if(void 0===this.st){this.st=new Set,void 0!==t.strings&&(this.nt=new Set(t.strings.join(" ").split(/\s/).filter(t=>""!==t)));for(const t in e)e[t]&&!this.nt?.has(t)&&this.st.add(t);return this.render(e)}const i=t.element.classList;for(const t of this.st)t in e||(i.remove(t),this.st.delete(t));for(const t in e){const o=!!e[t];o===this.st.has(t)||this.nt?.has(t)||(o?(i.add(t),this.st.add(t)):(i.remove(t),this.st.delete(t)))}return Q}}),Tt="cubic-bezier(0.2, 0, 0, 1)",zt="cubic-bezier(.3,0,0,1)",Lt="cubic-bezier(.3,0,.8,.15)";
/**
 * @license
 * Copyright 2021 Google LLC
 * SPDX-License-Identifier: Apache-2.0
 */var Nt;!function(t){t[t.INACTIVE=0]="INACTIVE",t[t.TOUCH_DELAY=1]="TOUCH_DELAY",t[t.HOLDING=2]="HOLDING",t[t.WAITING_FOR_CLICK=3]="WAITING_FOR_CLICK"}(Nt||(Nt={}));const Dt=["click","contextmenu","pointercancel","pointerdown","pointerenter","pointerleave","pointerup"],Bt=window.matchMedia("(forced-colors: active)");class Ut extends ft{constructor(){super(...arguments),this.disabled=!1,this.hovered=!1,this.pressed=!1,this.rippleSize="",this.rippleScale="",this.initialSize=0,this.state=Nt.INACTIVE,this.attachableController=new $t(this,this.onControlChange.bind(this))}get htmlFor(){return this.attachableController.htmlFor}set htmlFor(t){this.attachableController.htmlFor=t}get control(){return this.attachableController.control}set control(t){this.attachableController.control=t}attach(t){this.attachableController.attach(t)}detach(){this.attachableController.detach()}connectedCallback(){super.connectedCallback(),this.setAttribute("aria-hidden","true")}render(){const t={hovered:this.hovered,pressed:this.pressed};return J`<div class="surface ${Ot(t)}"></div>`}update(t){t.has("disabled")&&this.disabled&&(this.hovered=!1,this.pressed=!1),super.update(t)}handlePointerenter(t){this.shouldReactToEvent(t)&&(this.hovered=!0)}handlePointerleave(t){this.shouldReactToEvent(t)&&(this.hovered=!1,this.state!==Nt.INACTIVE&&this.endPressAnimation())}handlePointerup(t){if(this.shouldReactToEvent(t)){if(this.state!==Nt.HOLDING)return this.state===Nt.TOUCH_DELAY?(this.state=Nt.WAITING_FOR_CLICK,void this.startPressAnimation(this.rippleStartEvent)):void 0;this.state=Nt.WAITING_FOR_CLICK}}async handlePointerdown(t){if(this.shouldReactToEvent(t)){if(this.rippleStartEvent=t,!this.isTouch(t))return this.state=Nt.WAITING_FOR_CLICK,void this.startPressAnimation(t);this.state=Nt.TOUCH_DELAY,await new Promise(t=>{setTimeout(t,150)}),this.state===Nt.TOUCH_DELAY&&(this.state=Nt.HOLDING,this.startPressAnimation(t))}}handleClick(){this.disabled||(this.state!==Nt.WAITING_FOR_CLICK?this.state===Nt.INACTIVE&&(this.startPressAnimation(),this.endPressAnimation()):this.endPressAnimation())}handlePointercancel(t){this.shouldReactToEvent(t)&&this.endPressAnimation()}handleContextmenu(){this.disabled||this.endPressAnimation()}determineRippleSize(){const{height:t,width:e}=this.getBoundingClientRect(),i=Math.max(t,e),o=Math.max(.35*i,75),s=this.currentCSSZoom??1,r=Math.floor(.2*i/s),a=Math.sqrt(e**2+t**2)+10;this.initialSize=r;const n=(a+o)/r;this.rippleScale=""+n/s,this.rippleSize=`${r}px`}getNormalizedPointerEventCoords(t){const{scrollX:e,scrollY:i}=window,{left:o,top:s}=this.getBoundingClientRect(),r=e+o,a=i+s,{pageX:n,pageY:l}=t,d=this.currentCSSZoom??1;return{x:(n-r)/d,y:(l-a)/d}}getTranslationCoordinates(t){const{height:e,width:i}=this.getBoundingClientRect(),o=this.currentCSSZoom??1,s={x:(i/o-this.initialSize)/2,y:(e/o-this.initialSize)/2};let r;return r=t instanceof PointerEvent?this.getNormalizedPointerEventCoords(t):{x:i/o/2,y:e/o/2},r={x:r.x-this.initialSize/2,y:r.y-this.initialSize/2},{startPoint:r,endPoint:s}}startPressAnimation(t){if(!this.mdRoot)return;this.pressed=!0,this.growAnimation?.cancel(),this.determineRippleSize();const{startPoint:e,endPoint:i}=this.getTranslationCoordinates(t),o=`${e.x}px, ${e.y}px`,s=`${i.x}px, ${i.y}px`;this.growAnimation=this.mdRoot.animate({top:[0,0],left:[0,0],height:[this.rippleSize,this.rippleSize],width:[this.rippleSize,this.rippleSize],transform:[`translate(${o}) scale(1)`,`translate(${s}) scale(${this.rippleScale})`]},{pseudoElement:"::after",duration:450,easing:Tt,fill:"forwards"})}async endPressAnimation(){this.rippleStartEvent=void 0,this.state=Nt.INACTIVE;const t=this.growAnimation;let e=1/0;"number"==typeof t?.currentTime?e=t.currentTime:t?.currentTime&&(e=t.currentTime.to("ms").value),e>=225?this.pressed=!1:(await new Promise(t=>{setTimeout(t,225-e)}),this.growAnimation===t&&(this.pressed=!1))}shouldReactToEvent(t){if(this.disabled||!t.isPrimary)return!1;if(this.rippleStartEvent&&this.rippleStartEvent.pointerId!==t.pointerId)return!1;if("pointerenter"===t.type||"pointerleave"===t.type)return!this.isTouch(t);const e=1===t.buttons;return this.isTouch(t)||e}isTouch({pointerType:t}){return"touch"===t}async handleEvent(t){if(!Bt?.matches)switch(t.type){case"click":this.handleClick();break;case"contextmenu":this.handleContextmenu();break;case"pointercancel":this.handlePointercancel(t);break;case"pointerdown":await this.handlePointerdown(t);break;case"pointerenter":this.handlePointerenter(t);break;case"pointerleave":this.handlePointerleave(t);break;case"pointerup":this.handlePointerup(t)}}onControlChange(t,e){for(const i of Dt)t?.removeEventListener(i,this),e?.addEventListener(i,this)}}t([A({type:Boolean,reflect:!0})],Ut.prototype,"disabled",void 0),t([k()],Ut.prototype,"hovered",void 0),t([k()],Ut.prototype,"pressed",void 0),t([S(".surface")],Ut.prototype,"mdRoot",void 0);
/**
 * @license
 * Copyright 2024 Google LLC
 * SPDX-License-Identifier: Apache-2.0
 */
const Ht=n`:host{display:flex;margin:auto;pointer-events:none}:host([disabled]){display:none}@media(forced-colors: active){:host{display:none}}:host,.surface{border-radius:inherit;position:absolute;inset:0;overflow:hidden}.surface{-webkit-tap-highlight-color:rgba(0,0,0,0)}.surface::before,.surface::after{content:"";opacity:0;position:absolute}.surface::before{background-color:var(--md-ripple-hover-color, var(--md-sys-color-on-surface, #1d1b20));inset:0;transition:opacity 15ms linear,background-color 15ms linear}.surface::after{background:radial-gradient(closest-side, var(--md-ripple-pressed-color, var(--md-sys-color-on-surface, #1d1b20)) max(100% - 70px, 65%), transparent 100%);transform-origin:center center;transition:opacity 375ms linear}.hovered::before{background-color:var(--md-ripple-hover-color, var(--md-sys-color-on-surface, #1d1b20));opacity:var(--md-ripple-hover-opacity, 0.08)}.pressed::after{opacity:var(--md-ripple-pressed-opacity, 0.12);transition-duration:105ms}
`
/**
 * @license
 * Copyright 2022 Google LLC
 * SPDX-License-Identifier: Apache-2.0
 */;let Ft=class extends Ut{};Ft.styles=[Ht],Ft=t([e("md-ripple")],Ft);
/**
 * @license
 * Copyright 2020 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */
const Mt=Symbol.for(""),Wt=t=>{if(t?.r===Mt)return t?._$litStatic$},qt=(t,...e)=>({_$litStatic$:e.reduce((e,i,o)=>e+(t=>{if(void 0!==t._$litStatic$)return t._$litStatic$;throw Error(`Value passed to 'literal' function must be a 'literal' result: ${t}. Use 'unsafeStatic' to pass non-literal values, but\n            take care to ensure page security.`)})(i)+t[o+1],t[0]),r:Mt}),jt=new Map,Vt=(t=>(e,...i)=>{const o=i.length;let s,r;const a=[],n=[];let l,d=0,c=!1;for(;d<o;){for(l=e[d];d<o&&void 0!==(r=i[d],s=Wt(r));)l+=s+e[++d],c=!0;d!==o&&n.push(r),a.push(l),d++}if(d===o&&a.push(e[o]),c){const t=a.join("$$lit$$");void 0===(e=jt.get(t))&&(a.raw=a,jt.set(t,e=a)),i=n}return t(e,...i)})(J),Kt=["role","ariaAtomic","ariaAutoComplete","ariaBusy","ariaChecked","ariaColCount","ariaColIndex","ariaColSpan","ariaCurrent","ariaDisabled","ariaExpanded","ariaHasPopup","ariaHidden","ariaInvalid","ariaKeyShortcuts","ariaLabel","ariaLevel","ariaLive","ariaModal","ariaMultiLine","ariaMultiSelectable","ariaOrientation","ariaPlaceholder","ariaPosInSet","ariaPressed","ariaReadOnly","ariaRequired","ariaRoleDescription","ariaRowCount","ariaRowIndex","ariaRowSpan","ariaSelected","ariaSetSize","ariaSort","ariaValueMax","ariaValueMin","ariaValueNow","ariaValueText"],Gt=Kt.map(Xt);
/**
 * @license
 * Copyright 2023 Google LLC
 * SPDX-License-Identifier: Apache-2.0
 */function Yt(t){return Gt.includes(t)}function Xt(t){return t.replace("aria","aria-").replace(/Elements?/g,"").toLowerCase()}
/**
 * @license
 * Copyright 2023 Google LLC
 * SPDX-License-Identifier: Apache-2.0
 */const Zt=Symbol("privateIgnoreAttributeChangesFor");function Jt(t){var e;class i extends t{constructor(){super(...arguments),this[e]=new Set}attributeChangedCallback(t,e,i){if(!Yt(t))return void super.attributeChangedCallback(t,e,i);if(this[Zt].has(t))return;this[Zt].add(t),this.removeAttribute(t),this[Zt].delete(t);const o=te(t);null===i?delete this.dataset[o]:this.dataset[o]=i,this.requestUpdate(te(t),e)}getAttribute(t){return Yt(t)?super.getAttribute(Qt(t)):super.getAttribute(t)}removeAttribute(t){super.removeAttribute(t),Yt(t)&&(super.removeAttribute(Qt(t)),this.requestUpdate())}}return e=Zt,function(t){for(const e of Kt){const i=Xt(e),o=Qt(i),s=te(i);t.createProperty(e,{attribute:i,noAccessor:!0}),t.createProperty(Symbol(o),{attribute:o,noAccessor:!0}),Object.defineProperty(t.prototype,e,{configurable:!0,enumerable:!0,get(){return this.dataset[s]??null},set(t){const i=this.dataset[s]??null;t!==i&&(null===t?delete this.dataset[s]:this.dataset[s]=t,this.requestUpdate(e,i))}})}}(i),i}function Qt(t){return`data-${t}`}function te(t){return t.replace(/-\w/,t=>t[1].toUpperCase())}
/**
 * @license
 * Copyright 2023 Google LLC
 * SPDX-License-Identifier: Apache-2.0
 */const ee=Symbol("internals"),ie=Symbol("privateInternals");function oe(t){return class extends t{get[ee](){return this[ie]||(this[ie]=this.attachInternals()),this[ie]}}}
/**
 * @license
 * Copyright 2023 Google LLC
 * SPDX-License-Identifier: Apache-2.0
 */function se(t){t.addInitializer(t=>{const e=t;e.addEventListener("click",async t=>{const{type:i,[ee]:o}=e,{form:s}=o;s&&"button"!==i&&(await new Promise(t=>{setTimeout(t)}),t.defaultPrevented||("reset"!==i?(s.addEventListener("submit",t=>{Object.defineProperty(t,"submitter",{configurable:!0,enumerable:!0,get:()=>e})},{capture:!0,once:!0}),o.setFormValue(e.value),s.requestSubmit()):s.reset()))})})}
/**
 * @license
 * Copyright 2022 Google LLC
 * SPDX-License-Identifier: Apache-2.0
 */function re(t,e=!0){return e&&"rtl"===getComputedStyle(t).getPropertyValue("direction").trim()}
/**
 * @license
 * Copyright 2018 Google LLC
 * SPDX-License-Identifier: Apache-2.0
 */const ae=Jt(oe(ft));class ne extends ae{get name(){return this.getAttribute("name")??""}set name(t){this.setAttribute("name",t)}get form(){return this[ee].form}get labels(){return this[ee].labels}constructor(){super(),this.disabled=!1,this.softDisabled=!1,this.flipIconInRtl=!1,this.href="",this.download="",this.target="",this.ariaLabelSelected="",this.toggle=!1,this.selected=!1,this.type="submit",this.value="",this.flipIcon=re(this,this.flipIconInRtl),this.addEventListener("click",this.handleClick.bind(this))}willUpdate(){this.href&&(this.disabled=!1,this.softDisabled=!1)}render(){const t=this.href?qt`div`:qt`button`,{ariaLabel:e,ariaHasPopup:i,ariaExpanded:o}=this,s=e&&this.ariaLabelSelected,r=this.toggle?this.selected:tt;let a=tt;return this.href||(a=s&&this.selected?this.ariaLabelSelected:e),Vt`<${t}
        class="icon-button ${Ot(this.getRenderClasses())}"
        id="button"
        aria-label="${a||tt}"
        aria-haspopup="${!this.href&&i||tt}"
        aria-expanded="${!this.href&&o||tt}"
        aria-pressed="${r}"
        aria-disabled=${!this.href&&this.softDisabled||tt}
        ?disabled="${!this.href&&this.disabled}"
        @click="${this.handleClickOnChild}">
        ${this.renderFocusRing()}
        ${this.renderRipple()}
        ${this.selected?tt:this.renderIcon()}
        ${this.selected?this.renderSelectedIcon():tt}
        ${this.href?this.renderLink():this.renderTouchTarget()}
  </${t}>`}renderLink(){const{ariaLabel:t}=this;return J`
      <a
        class="link"
        id="link"
        href="${this.href}"
        download="${this.download||tt}"
        target="${this.target||tt}"
        aria-label="${t||tt}">
        ${this.renderTouchTarget()}
      </a>
    `}getRenderClasses(){return{"flip-icon":this.flipIcon,selected:this.toggle&&this.selected}}renderIcon(){return J`<span class="icon"><slot></slot></span>`}renderSelectedIcon(){return J`<span class="icon icon--selected"
      ><slot name="selected"><slot></slot></slot
    ></span>`}renderTouchTarget(){return J`<span class="touch"></span>`}renderFocusRing(){return J`<md-focus-ring
      part="focus-ring"
      for=${this.href?"link":"button"}></md-focus-ring>`}renderRipple(){const t=!this.href&&(this.disabled||this.softDisabled);return J`<md-ripple
      for=${this.href?"link":tt}
      ?disabled="${t}"></md-ripple>`}connectedCallback(){this.flipIcon=re(this,this.flipIconInRtl),super.connectedCallback()}handleClick(t){if(!this.href&&this.softDisabled)return t.stopImmediatePropagation(),void t.preventDefault()}async handleClickOnChild(t){await 0,!this.toggle||this.disabled||this.softDisabled||t.defaultPrevented||(this.selected=!this.selected,this.dispatchEvent(new InputEvent("input",{bubbles:!0,composed:!0})),this.dispatchEvent(new Event("change",{bubbles:!0})))}}se(ne),ne.formAssociated=!0,ne.shadowRootOptions={mode:"open",delegatesFocus:!0},t([A({type:Boolean,reflect:!0})],ne.prototype,"disabled",void 0),t([A({type:Boolean,attribute:"soft-disabled",reflect:!0})],ne.prototype,"softDisabled",void 0),t([A({type:Boolean,attribute:"flip-icon-in-rtl"})],ne.prototype,"flipIconInRtl",void 0),t([A()],ne.prototype,"href",void 0),t([A()],ne.prototype,"download",void 0),t([A()],ne.prototype,"target",void 0),t([A({attribute:"aria-label-selected"})],ne.prototype,"ariaLabelSelected",void 0),t([A({type:Boolean})],ne.prototype,"toggle",void 0),t([A({type:Boolean,reflect:!0})],ne.prototype,"selected",void 0),t([A()],ne.prototype,"type",void 0),t([A({reflect:!0})],ne.prototype,"value",void 0),t([k()],ne.prototype,"flipIcon",void 0);
/**
 * @license
 * Copyright 2024 Google LLC
 * SPDX-License-Identifier: Apache-2.0
 */
const le=n`:host{display:inline-flex;outline:none;-webkit-tap-highlight-color:rgba(0,0,0,0);height:var(--_container-height);width:var(--_container-width);justify-content:center}:host([touch-target=wrapper]){margin:max(0px,(48px - var(--_container-height))/2) max(0px,(48px - var(--_container-width))/2)}md-focus-ring{--md-focus-ring-shape-start-start: var(--_container-shape-start-start);--md-focus-ring-shape-start-end: var(--_container-shape-start-end);--md-focus-ring-shape-end-end: var(--_container-shape-end-end);--md-focus-ring-shape-end-start: var(--_container-shape-end-start)}:host(:is([disabled],[soft-disabled])){pointer-events:none}.icon-button{place-items:center;background:none;border:none;box-sizing:border-box;cursor:pointer;display:flex;place-content:center;outline:none;padding:0;position:relative;text-decoration:none;user-select:none;z-index:0;flex:1;border-start-start-radius:var(--_container-shape-start-start);border-start-end-radius:var(--_container-shape-start-end);border-end-start-radius:var(--_container-shape-end-start);border-end-end-radius:var(--_container-shape-end-end)}.icon ::slotted(*){font-size:var(--_icon-size);height:var(--_icon-size);width:var(--_icon-size);font-weight:inherit}md-ripple{z-index:-1;border-start-start-radius:var(--_container-shape-start-start);border-start-end-radius:var(--_container-shape-start-end);border-end-start-radius:var(--_container-shape-end-start);border-end-end-radius:var(--_container-shape-end-end)}.flip-icon .icon{transform:scaleX(-1)}.icon{display:inline-flex}.link{display:grid;height:100%;outline:none;place-items:center;position:absolute;width:100%}.touch{position:absolute;height:max(48px,100%);width:max(48px,100%)}:host([touch-target=none]) .touch{display:none}@media(forced-colors: active){:host(:is([disabled],[soft-disabled])){--_disabled-icon-color: GrayText;--_disabled-icon-opacity: 1}}
`,de=n`:host{--_disabled-icon-color: var(--md-icon-button-disabled-icon-color, var(--md-sys-color-on-surface, #1d1b20));--_disabled-icon-opacity: var(--md-icon-button-disabled-icon-opacity, 0.38);--_icon-size: var(--md-icon-button-icon-size, 24px);--_selected-focus-icon-color: var(--md-icon-button-selected-focus-icon-color, var(--md-sys-color-primary, #6750a4));--_selected-hover-icon-color: var(--md-icon-button-selected-hover-icon-color, var(--md-sys-color-primary, #6750a4));--_selected-hover-state-layer-color: var(--md-icon-button-selected-hover-state-layer-color, var(--md-sys-color-primary, #6750a4));--_selected-hover-state-layer-opacity: var(--md-icon-button-selected-hover-state-layer-opacity, 0.08);--_selected-icon-color: var(--md-icon-button-selected-icon-color, var(--md-sys-color-primary, #6750a4));--_selected-pressed-icon-color: var(--md-icon-button-selected-pressed-icon-color, var(--md-sys-color-primary, #6750a4));--_selected-pressed-state-layer-color: var(--md-icon-button-selected-pressed-state-layer-color, var(--md-sys-color-primary, #6750a4));--_selected-pressed-state-layer-opacity: var(--md-icon-button-selected-pressed-state-layer-opacity, 0.12);--_state-layer-height: var(--md-icon-button-state-layer-height, 40px);--_state-layer-shape: var(--md-icon-button-state-layer-shape, var(--md-sys-shape-corner-full, 9999px));--_state-layer-width: var(--md-icon-button-state-layer-width, 40px);--_focus-icon-color: var(--md-icon-button-focus-icon-color, var(--md-sys-color-on-surface-variant, #49454f));--_hover-icon-color: var(--md-icon-button-hover-icon-color, var(--md-sys-color-on-surface-variant, #49454f));--_hover-state-layer-color: var(--md-icon-button-hover-state-layer-color, var(--md-sys-color-on-surface-variant, #49454f));--_hover-state-layer-opacity: var(--md-icon-button-hover-state-layer-opacity, 0.08);--_icon-color: var(--md-icon-button-icon-color, var(--md-sys-color-on-surface-variant, #49454f));--_pressed-icon-color: var(--md-icon-button-pressed-icon-color, var(--md-sys-color-on-surface-variant, #49454f));--_pressed-state-layer-color: var(--md-icon-button-pressed-state-layer-color, var(--md-sys-color-on-surface-variant, #49454f));--_pressed-state-layer-opacity: var(--md-icon-button-pressed-state-layer-opacity, 0.12);--_container-shape-start-start: 0;--_container-shape-start-end: 0;--_container-shape-end-end: 0;--_container-shape-end-start: 0;--_container-height: 0;--_container-width: 0;height:var(--_state-layer-height);width:var(--_state-layer-width)}:host([touch-target=wrapper]){margin:max(0px,(48px - var(--_state-layer-height))/2) max(0px,(48px - var(--_state-layer-width))/2)}md-focus-ring{--md-focus-ring-shape-start-start: var(--_state-layer-shape);--md-focus-ring-shape-start-end: var(--_state-layer-shape);--md-focus-ring-shape-end-end: var(--_state-layer-shape);--md-focus-ring-shape-end-start: var(--_state-layer-shape)}.standard{background-color:rgba(0,0,0,0);color:var(--_icon-color);--md-ripple-hover-color: var(--_hover-state-layer-color);--md-ripple-hover-opacity: var(--_hover-state-layer-opacity);--md-ripple-pressed-color: var(--_pressed-state-layer-color);--md-ripple-pressed-opacity: var(--_pressed-state-layer-opacity)}.standard:hover{color:var(--_hover-icon-color)}.standard:focus{color:var(--_focus-icon-color)}.standard:active{color:var(--_pressed-icon-color)}.standard:is(:disabled,[aria-disabled=true]){color:var(--_disabled-icon-color)}md-ripple{border-radius:var(--_state-layer-shape)}.standard:is(:disabled,[aria-disabled=true]){opacity:var(--_disabled-icon-opacity)}.selected:not(:disabled,[aria-disabled=true]){color:var(--_selected-icon-color)}.selected:not(:disabled,[aria-disabled=true]):hover{color:var(--_selected-hover-icon-color)}.selected:not(:disabled,[aria-disabled=true]):focus{color:var(--_selected-focus-icon-color)}.selected:not(:disabled,[aria-disabled=true]):active{color:var(--_selected-pressed-icon-color)}.selected{--md-ripple-hover-color: var(--_selected-hover-state-layer-color);--md-ripple-hover-opacity: var(--_selected-hover-state-layer-opacity);--md-ripple-pressed-color: var(--_selected-pressed-state-layer-color);--md-ripple-pressed-opacity: var(--_selected-pressed-state-layer-opacity)}
`
/**
 * @license
 * Copyright 2021 Google LLC
 * SPDX-License-Identifier: Apache-2.0
 */;
/**
 * @license
 * Copyright 2024 Google LLC
 * SPDX-License-Identifier: Apache-2.0
 */let ce=class extends ne{getRenderClasses(){return{...super.getRenderClasses(),standard:!0}}};ce.styles=[le,de],ce=t([e("md-icon-button")],ce);
/**
 * @license
 * Copyright 2022 Google LLC
 * SPDX-License-Identifier: Apache-2.0
 */
class he extends ft{connectedCallback(){super.connectedCallback(),this.setAttribute("aria-hidden","true")}render(){return J`<span class="shadow"></span>`}}
/**
 * @license
 * Copyright 2024 Google LLC
 * SPDX-License-Identifier: Apache-2.0
 */const pe=n`:host,.shadow,.shadow::before,.shadow::after{border-radius:inherit;inset:0;position:absolute;transition-duration:inherit;transition-property:inherit;transition-timing-function:inherit}:host{display:flex;pointer-events:none;transition-property:box-shadow,opacity}.shadow::before,.shadow::after{content:"";transition-property:box-shadow,opacity;--_level: var(--md-elevation-level, 0);--_shadow-color: var(--md-elevation-shadow-color, var(--md-sys-color-shadow, #000))}.shadow::before{box-shadow:0px calc(1px*(clamp(0,var(--_level),1) + clamp(0,var(--_level) - 3,1) + 2*clamp(0,var(--_level) - 4,1))) calc(1px*(2*clamp(0,var(--_level),1) + clamp(0,var(--_level) - 2,1) + clamp(0,var(--_level) - 4,1))) 0px var(--_shadow-color);opacity:.3}.shadow::after{box-shadow:0px calc(1px*(clamp(0,var(--_level),1) + clamp(0,var(--_level) - 1,1) + 2*clamp(0,var(--_level) - 2,3))) calc(1px*(3*clamp(0,var(--_level),2) + 2*clamp(0,var(--_level) - 2,3))) calc(1px*(clamp(0,var(--_level),4) + 2*clamp(0,var(--_level) - 4,1))) var(--_shadow-color);opacity:.15}
`
/**
 * @license
 * Copyright 2022 Google LLC
 * SPDX-License-Identifier: Apache-2.0
 */;let ue=class extends he{};ue.styles=[pe],ue=t([e("md-elevation")],ue);
/**
 * @license
 * Copyright 2018 Google LLC
 * SPDX-License-Identifier: BSD-3-Clause
 */
const me="important",ve=" !"+me,fe=Rt(class extends Pt{constructor(t){if(super(t),t.type!==St||"style"!==t.name||t.strings?.length>2)throw Error("The `styleMap` directive must be used in the `style` attribute and must be the only part in the attribute.")}render(t){return Object.keys(t).reduce((e,i)=>{const o=t[i];return null==o?e:e+`${i=i.includes("-")?i:i.replace(/(?:^(webkit|moz|ms|o)|)(?=[A-Z])/g,"-$&").toLowerCase()}:${o};`},"")}update(t,[e]){const{style:i}=t.element;if(void 0===this.ft)return this.ft=new Set(Object.keys(e)),this.render(e);for(const t of this.ft)null==e[t]&&(this.ft.delete(t),t.includes("-")?i.removeProperty(t):i[t]=null);for(const t in e){const o=e[t];if(null!=o){this.ft.add(t);const e="string"==typeof o&&o.endsWith(ve);t.includes("-")||e?i.setProperty(t,e?o.slice(0,-11):o,e?me:""):i[t]=o}}return Q}});
/**
 * @license
 * Copyright 2023 Google LLC
 * SPDX-License-Identifier: Apache-2.0
 */function be(t,e=Ce){const i=xe(t,e);return i&&(i.tabIndex=0,i.focus()),i}function ye(t,e=Ce){const i=_e(t,e);return i&&(i.tabIndex=0,i.focus()),i}function ge(t,e=Ce){for(let i=0;i<t.length;i++){const o=t[i];if(0===o.tabIndex&&e(o))return{item:o,index:i}}return null}function xe(t,e=Ce){for(const i of t)if(e(i))return i;return null}function _e(t,e=Ce){for(let i=t.length-1;i>=0;i--){const o=t[i];if(e(o))return o}return null}function we(t,e,i=Ce,o=!0){if(e){const s=function(t,e,i=Ce,o=!0){for(let s=1;s<t.length;s++){const r=(s+e)%t.length;if(r<e&&!o)return null;const a=t[r];if(i(a))return a}return t[e]?t[e]:null}(t,e.index,i,o);return s&&(s.tabIndex=0,s.focus()),s}return be(t,i)}function $e(t,e,i=Ce,o=!0){if(e){const s=function(t,e,i=Ce,o=!0){for(let s=1;s<t.length;s++){const r=(e-s+t.length)%t.length;if(r>e&&!o)return null;const a=t[r];if(i(a))return a}return t[e]?t[e]:null}(t,e.index,i,o);return s&&(s.tabIndex=0,s.focus()),s}return ye(t,i)}function Ce(t){return!t.disabled}
/**
 * @license
 * Copyright 2023 Google LLC
 * SPDX-License-Identifier: Apache-2.0
 */const Ee={ArrowDown:"ArrowDown",ArrowLeft:"ArrowLeft",ArrowUp:"ArrowUp",ArrowRight:"ArrowRight",Home:"Home",End:"End"};class Ae{constructor(t){this.handleKeydown=t=>{const e=t.key;if(t.defaultPrevented||!this.isNavigableKey(e))return;const i=this.items;if(!i.length)return;const o=ge(i,this.isActivatable);t.preventDefault();const s=this.isRtl();let r=null;switch(e){case Ee.ArrowDown:case s?Ee.ArrowLeft:Ee.ArrowRight:r=we(i,o,this.isActivatable,this.wrapNavigation());break;case Ee.ArrowUp:case s?Ee.ArrowRight:Ee.ArrowLeft:r=$e(i,o,this.isActivatable,this.wrapNavigation());break;case Ee.Home:r=be(i,this.isActivatable);break;case Ee.End:r=ye(i,this.isActivatable)}r&&o&&o.item!==r&&(o.item.tabIndex=-1)},this.onDeactivateItems=()=>{const t=this.items;for(const e of t)this.deactivateItem(e)},this.onRequestActivation=t=>{this.onDeactivateItems();const e=t.target;this.activateItem(e),e.focus()},this.onSlotchange=()=>{const t=this.items;let e=!1;for(const i of t){!(!i.disabled&&i.tabIndex>-1)||e?i.tabIndex=-1:(e=!0,i.tabIndex=0)}if(e)return;const i=xe(t,this.isActivatable);i&&(i.tabIndex=0)};const{isItem:e,getPossibleItems:i,isRtl:o,deactivateItem:s,activateItem:r,isNavigableKey:a,isActivatable:n,wrapNavigation:l}=t;this.isItem=e,this.getPossibleItems=i,this.isRtl=o,this.deactivateItem=s,this.activateItem=r,this.isNavigableKey=a,this.isActivatable=n,this.wrapNavigation=l??(()=>!0)}get items(){const t=this.getPossibleItems(),e=[];for(const i of t){if(this.isItem(i)){e.push(i);continue}const t=i.item;t&&this.isItem(t)&&e.push(t)}return e}activateNextItem(){const t=this.items,e=ge(t,this.isActivatable);return e&&(e.item.tabIndex=-1),we(t,e,this.isActivatable,this.wrapNavigation())}activatePreviousItem(){const t=this.items,e=ge(t,this.isActivatable);return e&&(e.item.tabIndex=-1),$e(t,e,this.isActivatable,this.wrapNavigation())}}
/**
 * @license
 * Copyright 2023 Google LLC
 * SPDX-License-Identifier: Apache-2.0
 */const ke=function(t,e){return new CustomEvent("close-menu",{bubbles:!0,composed:!0,detail:{initiator:t,reason:e,itemPath:[t]}})},Ie="click-selection",Se="keydown",Re={ESCAPE:"Escape",SPACE:"Space",ENTER:"Enter"};function Pe(t){return Object.values(Re).some(e=>e===t)}function Oe(t,e){const i=new Event("md-contains",{bubbles:!0,composed:!0});let o=[];const s=t=>{o=t.composedPath()};e.addEventListener("md-contains",s),t.dispatchEvent(i),e.removeEventListener("md-contains",s);return o.length>0}const Te="none",ze="list-root",Le="first-item",Ne="last-item",De="end-start",Be="start-start";
/**
 * @license
 * Copyright 2023 Google LLC
 * SPDX-License-Identifier: Apache-2.0
 */class Ue{constructor(t,e){this.host=t,this.getProperties=e,this.surfaceStylesInternal={display:"none"},this.lastValues={isOpen:!1},this.host.addController(this)}get surfaceStyles(){return this.surfaceStylesInternal}async position(){const{surfaceEl:t,anchorEl:e,anchorCorner:i,surfaceCorner:o,positioning:s,xOffset:r,yOffset:a,disableBlockFlip:n,disableInlineFlip:l,repositionStrategy:d}=this.getProperties(),c=i.toLowerCase().trim(),h=o.toLowerCase().trim();if(!t||!e)return;const p=window.innerWidth,u=window.innerHeight,m=document.createElement("div");m.style.opacity="0",m.style.position="fixed",m.style.display="block",m.style.inset="0",document.body.appendChild(m);const v=m.getBoundingClientRect();m.remove();const f=window.innerHeight-v.bottom,b=window.innerWidth-v.right;this.surfaceStylesInternal={display:"block",opacity:"0"},this.host.requestUpdate(),await this.host.updateComplete,t.popover&&t.isConnected&&t.showPopover();const y=t.getSurfacePositionClientRect?t.getSurfacePositionClientRect():t.getBoundingClientRect(),g=e.getSurfacePositionClientRect?e.getSurfacePositionClientRect():e.getBoundingClientRect(),[x,_]=h.split("-"),[w,$]=c.split("-"),C="ltr"===getComputedStyle(t).direction;let{blockInset:E,blockOutOfBoundsCorrection:A,surfaceBlockProperty:k}=this.calculateBlock({surfaceRect:y,anchorRect:g,anchorBlock:w,surfaceBlock:x,yOffset:a,positioning:s,windowInnerHeight:u,blockScrollbarHeight:f});if(A&&!n){const t="start"===x?"end":"start",e="start"===w?"end":"start",i=this.calculateBlock({surfaceRect:y,anchorRect:g,anchorBlock:e,surfaceBlock:t,yOffset:a,positioning:s,windowInnerHeight:u,blockScrollbarHeight:f});A>i.blockOutOfBoundsCorrection&&(E=i.blockInset,A=i.blockOutOfBoundsCorrection,k=i.surfaceBlockProperty)}let{inlineInset:I,inlineOutOfBoundsCorrection:S,surfaceInlineProperty:R}=this.calculateInline({surfaceRect:y,anchorRect:g,anchorInline:$,surfaceInline:_,xOffset:r,positioning:s,isLTR:C,windowInnerWidth:p,inlineScrollbarWidth:b});if(S&&!l){const t="start"===_?"end":"start",e="start"===$?"end":"start",i=this.calculateInline({surfaceRect:y,anchorRect:g,anchorInline:e,surfaceInline:t,xOffset:r,positioning:s,isLTR:C,windowInnerWidth:p,inlineScrollbarWidth:b});Math.abs(S)>Math.abs(i.inlineOutOfBoundsCorrection)&&(I=i.inlineInset,S=i.inlineOutOfBoundsCorrection,R=i.surfaceInlineProperty)}"move"===d&&(E-=A,I-=S),this.surfaceStylesInternal={display:"block",opacity:"1",[k]:`${E}px`,[R]:`${I}px`},"resize"===d&&(A&&(this.surfaceStylesInternal.height=y.height-A+"px"),S&&(this.surfaceStylesInternal.width=y.width-S+"px")),this.host.requestUpdate()}calculateBlock(t){const{surfaceRect:e,anchorRect:i,anchorBlock:o,surfaceBlock:s,yOffset:r,positioning:a,windowInnerHeight:n,blockScrollbarHeight:l}=t,d="fixed"===a||"document"===a?1:0,c="document"===a?1:0,h="start"===s?1:0,p="end"===s?1:0,u=(o!==s?1:0)*i.height+r,m=h*i.top+p*(n-i.bottom-l);return{blockInset:d*m+c*(h*window.scrollY-p*window.scrollY)+u,blockOutOfBoundsCorrection:Math.abs(Math.min(0,n-m-u-e.height)),surfaceBlockProperty:"start"===s?"inset-block-start":"inset-block-end"}}calculateInline(t){const{isLTR:e,surfaceInline:i,anchorInline:o,anchorRect:s,surfaceRect:r,xOffset:a,positioning:n,windowInnerWidth:l,inlineScrollbarWidth:d}=t,c="fixed"===n||"document"===n?1:0,h="document"===n?1:0,p=e?1:0,u=e?0:1,m="start"===i?1:0,v="end"===i?1:0,f=(o!==i?1:0)*s.width+a,b=p*(m*s.left+v*(l-s.right-d))+u*(m*(l-s.right-d)+v*s.left);let y="start"===i?"inset-inline-start":"inset-inline-end";return"document"!==n&&"fixed"!==n||(y="start"===i&&e||"end"===i&&!e?"left":"right"),{inlineInset:c*b+f+h*(p*(m*window.scrollX-v*window.scrollX)+u*(v*window.scrollX-m*window.scrollX)),inlineOutOfBoundsCorrection:Math.abs(Math.min(0,l-b-f-r.width)),surfaceInlineProperty:y}}hostUpdate(){this.onUpdate()}hostUpdated(){this.onUpdate()}async onUpdate(){const t=this.getProperties();let e=!1;for(const[i,o]of Object.entries(t))if(e=e||o!==this.lastValues[i],e)break;const i=this.lastValues.isOpen!==t.isOpen,o=!!t.anchorEl,s=!!t.surfaceEl;e&&o&&s&&(this.lastValues.isOpen=t.isOpen,t.isOpen?(this.lastValues=t,await this.position(),t.onOpen()):i&&(await t.beforeClose(),this.close(),t.onClose()))}close(){this.surfaceStylesInternal={display:"none"},this.host.requestUpdate();const t=this.getProperties().surfaceEl;t?.popover&&t?.isConnected&&t.hidePopover()}}
/**
 * @license
 * Copyright 2023 Google LLC
 * SPDX-License-Identifier: Apache-2.0
 */const He=0,Fe=1,Me=2;class We{constructor(t){this.getProperties=t,this.typeaheadRecords=[],this.typaheadBuffer="",this.cancelTypeaheadTimeout=0,this.isTypingAhead=!1,this.lastActiveRecord=null,this.onKeydown=t=>{this.isTypingAhead?this.typeahead(t):this.beginTypeahead(t)},this.endTypeahead=()=>{this.isTypingAhead=!1,this.typaheadBuffer="",this.typeaheadRecords=[]}}get items(){return this.getProperties().getItems()}get active(){return this.getProperties().active}beginTypeahead(t){this.active&&("Space"===t.code||"Enter"===t.code||t.code.startsWith("Arrow")||"Escape"===t.code||(this.isTypingAhead=!0,this.typeaheadRecords=this.items.map((t,e)=>[e,t,t.typeaheadText.trim().toLowerCase()]),this.lastActiveRecord=this.typeaheadRecords.find(t=>0===t[Fe].tabIndex)??null,this.lastActiveRecord&&(this.lastActiveRecord[Fe].tabIndex=-1),this.typeahead(t)))}typeahead(t){if(t.defaultPrevented)return;if(clearTimeout(this.cancelTypeaheadTimeout),"Enter"===t.code||t.code.startsWith("Arrow")||"Escape"===t.code)return this.endTypeahead(),void(this.lastActiveRecord&&(this.lastActiveRecord[Fe].tabIndex=-1));"Space"===t.code&&t.preventDefault(),this.cancelTypeaheadTimeout=setTimeout(this.endTypeahead,this.getProperties().typeaheadBufferTime),this.typaheadBuffer+=t.key.toLowerCase();const e=this.lastActiveRecord?this.lastActiveRecord[He]:-1,i=this.typeaheadRecords.length,o=t=>(t[He]+i-e)%i,s=this.typeaheadRecords.filter(t=>!t[Fe].disabled&&t[Me].startsWith(this.typaheadBuffer)).sort((t,e)=>o(t)-o(e));if(0===s.length)return clearTimeout(this.cancelTypeaheadTimeout),this.lastActiveRecord&&(this.lastActiveRecord[Fe].tabIndex=-1),void this.endTypeahead();const r=1===this.typaheadBuffer.length;let a;a=this.lastActiveRecord===s[0]&&r?s[1]??s[0]:s[0],this.lastActiveRecord&&(this.lastActiveRecord[Fe].tabIndex=-1),this.lastActiveRecord=a,a[Fe].tabIndex=0,a[Fe].focus()}}
/**
 * @license
 * Copyright 2023 Google LLC
 * SPDX-License-Identifier: Apache-2.0
 */const qe=new Set([Ee.ArrowDown,Ee.ArrowUp,Ee.Home,Ee.End]),je=new Set([Ee.ArrowLeft,Ee.ArrowRight,...qe]);class Ve extends ft{get openDirection(){return"start"===this.menuCorner.split("-")[0]?"DOWN":"UP"}get anchorElement(){return this.anchor?this.getRootNode().querySelector(`#${this.anchor}`):this.currentAnchorElement}set anchorElement(t){this.currentAnchorElement=t,this.requestUpdate("anchorElement")}constructor(){super(),this.anchor="",this.positioning="absolute",this.quick=!1,this.hasOverflow=!1,this.open=!1,this.xOffset=0,this.yOffset=0,this.noHorizontalFlip=!1,this.noVerticalFlip=!1,this.typeaheadDelay=200,this.anchorCorner=De,this.menuCorner=Be,this.stayOpenOnOutsideClick=!1,this.stayOpenOnFocusout=!1,this.skipRestoreFocus=!1,this.defaultFocus=Le,this.noNavigationWrap=!1,this.typeaheadActive=!0,this.isSubmenu=!1,this.pointerPath=[],this.isRepositioning=!1,this.openCloseAnimationSignal=function(){let t=null;return{start:()=>(t?.abort(),t=new AbortController,t.signal),finish(){t=null}}}
/**
 * @license
 * Copyright 2022 Google LLC
 * SPDX-License-Identifier: Apache-2.0
 */(),this.listController=new Ae({isItem:t=>t.hasAttribute("md-menu-item"),getPossibleItems:()=>this.slotItems,isRtl:()=>"rtl"===getComputedStyle(this).direction,deactivateItem:t=>{t.selected=!1,t.tabIndex=-1},activateItem:t=>{t.selected=!0,t.tabIndex=0},isNavigableKey:t=>{if(!this.isSubmenu)return je.has(t);return t===("rtl"===getComputedStyle(this).direction?Ee.ArrowLeft:Ee.ArrowRight)||qe.has(t)},wrapNavigation:()=>!this.noNavigationWrap}),this.lastFocusedElement=null,this.typeaheadController=new We(()=>({getItems:()=>this.items,typeaheadBufferTime:this.typeaheadDelay,active:this.typeaheadActive})),this.currentAnchorElement=null,this.internals=this.attachInternals(),this.menuPositionController=new Ue(this,()=>({anchorCorner:this.anchorCorner,surfaceCorner:this.menuCorner,surfaceEl:this.surfaceEl,anchorEl:this.anchorElement,positioning:"popover"===this.positioning?"document":this.positioning,isOpen:this.open,xOffset:this.xOffset,yOffset:this.yOffset,disableBlockFlip:this.noVerticalFlip,disableInlineFlip:this.noHorizontalFlip,onOpen:this.onOpened,beforeClose:this.beforeClose,onClose:this.onClosed,repositionStrategy:this.hasOverflow&&"popover"!==this.positioning?"move":"resize"})),this.onWindowResize=()=>{this.isRepositioning||"document"!==this.positioning&&"fixed"!==this.positioning&&"popover"!==this.positioning||(this.isRepositioning=!0,this.reposition(),this.isRepositioning=!1)},this.handleFocusout=async t=>{const e=this.anchorElement;if(this.stayOpenOnFocusout||!this.open||this.pointerPath.includes(e))return;if(t.relatedTarget){if(Oe(t.relatedTarget,this)||0!==this.pointerPath.length&&Oe(t.relatedTarget,e))return}else if(this.pointerPath.includes(this))return;const i=this.skipRestoreFocus;this.skipRestoreFocus=!0,this.close(),await this.updateComplete,this.skipRestoreFocus=i},this.onOpened=async()=>{this.lastFocusedElement=function(t=document){let e=t.activeElement;for(;e&&e?.shadowRoot?.activeElement;)e=e.shadowRoot.activeElement;return e}();const t=this.items,e=ge(t);e&&this.defaultFocus!==Te&&(e.item.tabIndex=-1);let i=!this.quick;switch(this.quick?this.dispatchEvent(new Event("opening")):i=!!await this.animateOpen(),this.defaultFocus){case Le:const e=xe(t);e&&(e.tabIndex=0,e.focus(),await e.updateComplete);break;case Ne:const i=_e(t);i&&(i.tabIndex=0,i.focus(),await i.updateComplete);break;case ze:this.focus()}i||this.dispatchEvent(new Event("opened"))},this.beforeClose=async()=>{this.open=!1,this.skipRestoreFocus||this.lastFocusedElement?.focus?.(),this.quick||await this.animateClose()},this.onClosed=()=>{this.quick&&(this.dispatchEvent(new Event("closing")),this.dispatchEvent(new Event("closed")))},this.onWindowPointerdown=t=>{this.pointerPath=t.composedPath()},this.onDocumentClick=t=>{if(!this.open)return;const e=t.composedPath();this.stayOpenOnOutsideClick||e.includes(this)||e.includes(this.anchorElement)||(this.open=!1)},this.internals.role="menu",this.addEventListener("keydown",this.handleKeydown),this.addEventListener("keydown",this.captureKeydown,{capture:!0}),this.addEventListener("focusout",this.handleFocusout)}get items(){return this.listController.items}willUpdate(t){t.has("open")&&(this.open?this.removeAttribute("aria-hidden"):this.setAttribute("aria-hidden","true"))}update(t){t.has("open")&&(this.open?this.setUpGlobalEventListeners():this.cleanUpGlobalEventListeners()),t.has("positioning")&&"popover"===this.positioning&&!this.showPopover&&(this.positioning="fixed"),super.update(t)}connectedCallback(){super.connectedCallback(),this.open&&this.setUpGlobalEventListeners()}disconnectedCallback(){super.disconnectedCallback(),this.cleanUpGlobalEventListeners()}getBoundingClientRect(){return this.surfaceEl?this.surfaceEl.getBoundingClientRect():super.getBoundingClientRect()}getClientRects(){return this.surfaceEl?this.surfaceEl.getClientRects():super.getClientRects()}render(){return this.renderSurface()}renderSurface(){return J`
      <div
        class="menu ${Ot(this.getSurfaceClasses())}"
        style=${fe(this.menuPositionController.surfaceStyles)}
        popover=${"popover"===this.positioning?"manual":tt}>
        ${this.renderElevation()}
        <div class="items">
          <div class="item-padding"> ${this.renderMenuItems()} </div>
        </div>
      </div>
    `}renderMenuItems(){return J`<slot
      @close-menu=${this.onCloseMenu}
      @deactivate-items=${this.onDeactivateItems}
      @request-activation=${this.onRequestActivation}
      @deactivate-typeahead=${this.handleDeactivateTypeahead}
      @activate-typeahead=${this.handleActivateTypeahead}
      @stay-open-on-focusout=${this.handleStayOpenOnFocusout}
      @close-on-focusout=${this.handleCloseOnFocusout}
      @slotchange=${this.listController.onSlotchange}></slot>`}renderElevation(){return J`<md-elevation part="elevation"></md-elevation>`}getSurfaceClasses(){return{open:this.open,fixed:"fixed"===this.positioning,"has-overflow":this.hasOverflow}}captureKeydown(t){t.target===this&&!t.defaultPrevented&&Pe(t.code)&&(t.preventDefault(),this.close()),this.typeaheadController.onKeydown(t)}async animateOpen(){const t=this.surfaceEl,e=this.slotEl;if(!t||!e)return!0;const i=this.openDirection;this.dispatchEvent(new Event("opening")),t.classList.toggle("animating",!0);const o=this.openCloseAnimationSignal.start(),s=t.offsetHeight,r="UP"===i,a=this.items,n=250/a.length,l=t.animate([{height:"0px"},{height:`${s}px`}],{duration:500,easing:zt}),d=e.animate([{transform:r?`translateY(-${s}px)`:""},{transform:""}],{duration:500,easing:zt}),c=t.animate([{opacity:0},{opacity:1}],50),h=[];for(let t=0;t<a.length;t++){const e=a[r?a.length-1-t:t],i=e.animate([{opacity:0},{opacity:1}],{duration:250,delay:n*t});e.classList.toggle("md-menu-hidden",!0),i.addEventListener("finish",()=>{e.classList.toggle("md-menu-hidden",!1)}),h.push([e,i])}let p=t=>{};const u=new Promise(t=>{p=t});return o.addEventListener("abort",()=>{l.cancel(),d.cancel(),c.cancel(),h.forEach(([t,e])=>{t.classList.toggle("md-menu-hidden",!1),e.cancel()}),p(!0)}),l.addEventListener("finish",()=>{t.classList.toggle("animating",!1),this.openCloseAnimationSignal.finish(),p(!1)}),await u}animateClose(){let t;const e=new Promise(e=>{t=e}),i=this.surfaceEl,o=this.slotEl;if(!i||!o)return t(!1),e;const s="UP"===this.openDirection;this.dispatchEvent(new Event("closing")),i.classList.toggle("animating",!0);const r=this.openCloseAnimationSignal.start(),a=i.offsetHeight,n=this.items,l=150,d=50/n.length,c=i.animate([{height:`${a}px`},{height:.35*a+"px"}],{duration:l,easing:Lt}),h=o.animate([{transform:""},{transform:s?`translateY(-${.65*a}px)`:""}],{duration:l,easing:Lt}),p=i.animate([{opacity:1},{opacity:0}],{duration:50,delay:100}),u=[];for(let t=0;t<n.length;t++){const e=n[s?t:n.length-1-t],i=e.animate([{opacity:1},{opacity:0}],{duration:50,delay:50+d*t});i.addEventListener("finish",()=>{e.classList.toggle("md-menu-hidden",!0)}),u.push([e,i])}return r.addEventListener("abort",()=>{c.cancel(),h.cancel(),p.cancel(),u.forEach(([t,e])=>{e.cancel(),t.classList.toggle("md-menu-hidden",!1)}),t(!1)}),c.addEventListener("finish",()=>{i.classList.toggle("animating",!1),u.forEach(([t])=>{t.classList.toggle("md-menu-hidden",!1)}),this.openCloseAnimationSignal.finish(),this.dispatchEvent(new Event("closed")),t(!0)}),e}handleKeydown(t){this.pointerPath=[],this.listController.handleKeydown(t)}setUpGlobalEventListeners(){document.addEventListener("click",this.onDocumentClick,{capture:!0}),window.addEventListener("pointerdown",this.onWindowPointerdown),document.addEventListener("resize",this.onWindowResize,{passive:!0}),window.addEventListener("resize",this.onWindowResize,{passive:!0})}cleanUpGlobalEventListeners(){document.removeEventListener("click",this.onDocumentClick,{capture:!0}),window.removeEventListener("pointerdown",this.onWindowPointerdown),document.removeEventListener("resize",this.onWindowResize),window.removeEventListener("resize",this.onWindowResize)}onCloseMenu(){this.close()}onDeactivateItems(t){t.stopPropagation(),this.listController.onDeactivateItems()}onRequestActivation(t){t.stopPropagation(),this.listController.onRequestActivation(t)}handleDeactivateTypeahead(t){t.stopPropagation(),this.typeaheadActive=!1}handleActivateTypeahead(t){t.stopPropagation(),this.typeaheadActive=!0}handleStayOpenOnFocusout(t){t.stopPropagation(),this.stayOpenOnFocusout=!0}handleCloseOnFocusout(t){t.stopPropagation(),this.stayOpenOnFocusout=!1}close(){this.open=!1;this.slotItems.forEach(t=>{t.close?.()})}show(){this.open=!0}activateNextItem(){return this.listController.activateNextItem()??null}activatePreviousItem(){return this.listController.activatePreviousItem()??null}reposition(){this.open&&this.menuPositionController.position()}}t([S(".menu")],Ve.prototype,"surfaceEl",void 0),t([S("slot")],Ve.prototype,"slotEl",void 0),t([A()],Ve.prototype,"anchor",void 0),t([A()],Ve.prototype,"positioning",void 0),t([A({type:Boolean})],Ve.prototype,"quick",void 0),t([A({type:Boolean,attribute:"has-overflow"})],Ve.prototype,"hasOverflow",void 0),t([A({type:Boolean,reflect:!0})],Ve.prototype,"open",void 0),t([A({type:Number,attribute:"x-offset"})],Ve.prototype,"xOffset",void 0),t([A({type:Number,attribute:"y-offset"})],Ve.prototype,"yOffset",void 0),t([A({type:Boolean,attribute:"no-horizontal-flip"})],Ve.prototype,"noHorizontalFlip",void 0),t([A({type:Boolean,attribute:"no-vertical-flip"})],Ve.prototype,"noVerticalFlip",void 0),t([A({type:Number,attribute:"typeahead-delay"})],Ve.prototype,"typeaheadDelay",void 0),t([A({attribute:"anchor-corner"})],Ve.prototype,"anchorCorner",void 0),t([A({attribute:"menu-corner"})],Ve.prototype,"menuCorner",void 0),t([A({type:Boolean,attribute:"stay-open-on-outside-click"})],Ve.prototype,"stayOpenOnOutsideClick",void 0),t([A({type:Boolean,attribute:"stay-open-on-focusout"})],Ve.prototype,"stayOpenOnFocusout",void 0),t([A({type:Boolean,attribute:"skip-restore-focus"})],Ve.prototype,"skipRestoreFocus",void 0),t([A({attribute:"default-focus"})],Ve.prototype,"defaultFocus",void 0),t([A({type:Boolean,attribute:"no-navigation-wrap"})],Ve.prototype,"noNavigationWrap",void 0),t([P({flatten:!0})],Ve.prototype,"slotItems",void 0),t([k()],Ve.prototype,"typeaheadActive",void 0);
/**
 * @license
 * Copyright 2024 Google LLC
 * SPDX-License-Identifier: Apache-2.0
 */
const Ke=n`:host{--md-elevation-level: var(--md-menu-container-elevation, 2);--md-elevation-shadow-color: var(--md-menu-container-shadow-color, var(--md-sys-color-shadow, #000));min-width:112px;color:unset;display:contents}md-focus-ring{--md-focus-ring-shape: var(--md-menu-container-shape, var(--md-sys-shape-corner-extra-small, 4px))}.menu{border-radius:var(--md-menu-container-shape, var(--md-sys-shape-corner-extra-small, 4px));display:none;inset:auto;border:none;padding:0px;overflow:visible;background-color:rgba(0,0,0,0);color:inherit;opacity:0;z-index:20;position:absolute;user-select:none;max-height:inherit;height:inherit;min-width:inherit;max-width:inherit;scrollbar-width:inherit}.menu::backdrop{display:none}.fixed{position:fixed}.items{display:block;list-style-type:none;margin:0;outline:none;box-sizing:border-box;background-color:var(--md-menu-container-color, var(--md-sys-color-surface-container, #f3edf7));height:inherit;max-height:inherit;overflow:auto;min-width:inherit;max-width:inherit;border-radius:inherit;scrollbar-width:inherit}.item-padding{padding-block:var(--md-menu-top-space, 8px) var(--md-menu-bottom-space, 8px)}.has-overflow:not([popover]) .items{overflow:visible}.has-overflow.animating .items,.animating .items{overflow:hidden}.has-overflow.animating .items{pointer-events:none}.animating ::slotted(.md-menu-hidden){opacity:0}slot{display:block;height:inherit;max-height:inherit}::slotted(:is(md-divider,[role=separator])){margin:8px 0}@media(forced-colors: active){.menu{border-style:solid;border-color:CanvasText;border-width:1px}}
`
/**
 * @license
 * Copyright 2022 Google LLC
 * SPDX-License-Identifier: Apache-2.0
 */;let Ge=class extends Ve{};Ge.styles=[Ke],Ge=t([e("md-menu")],Ge);
/**
 * @license
 * Copyright 2023 Google LLC
 * SPDX-License-Identifier: Apache-2.0
 */
class Ye extends ft{constructor(){super(...arguments),this.multiline=!1}render(){return J`
      <slot name="container"></slot>
      <slot class="non-text" name="start"></slot>
      <div class="text">
        <slot name="overline" @slotchange=${this.handleTextSlotChange}></slot>
        <slot
          class="default-slot"
          @slotchange=${this.handleTextSlotChange}></slot>
        <slot name="headline" @slotchange=${this.handleTextSlotChange}></slot>
        <slot
          name="supporting-text"
          @slotchange=${this.handleTextSlotChange}></slot>
      </div>
      <slot class="non-text" name="trailing-supporting-text"></slot>
      <slot class="non-text" name="end"></slot>
    `}handleTextSlotChange(){let t=!1,e=0;for(const i of this.textSlots)if(Xe(i)&&(e+=1),e>1){t=!0;break}this.multiline=t}}function Xe(t){for(const e of t.assignedNodes({flatten:!0})){const t=e.nodeType===Node.ELEMENT_NODE,i=e.nodeType===Node.TEXT_NODE&&e.textContent?.match(/\S/);if(t||i)return!0}return!1}
/**
 * @license
 * Copyright 2024 Google LLC
 * SPDX-License-Identifier: Apache-2.0
 */t([A({type:Boolean,reflect:!0})],Ye.prototype,"multiline",void 0),t([function(t){return(e,i)=>I(e,i,{get(){return(this.renderRoot??(R??=document.createDocumentFragment())).querySelectorAll(t)}})}(".text slot")],Ye.prototype,"textSlots",void 0);const Ze=n`:host{color:var(--md-sys-color-on-surface, #1d1b20);font-family:var(--md-sys-typescale-body-large-font, var(--md-ref-typeface-plain, Roboto));font-size:var(--md-sys-typescale-body-large-size, 1rem);font-weight:var(--md-sys-typescale-body-large-weight, var(--md-ref-typeface-weight-regular, 400));line-height:var(--md-sys-typescale-body-large-line-height, 1.5rem);align-items:center;box-sizing:border-box;display:flex;gap:16px;min-height:56px;overflow:hidden;padding:12px 16px;position:relative;text-overflow:ellipsis}:host([multiline]){min-height:72px}[name=overline]{color:var(--md-sys-color-on-surface-variant, #49454f);font-family:var(--md-sys-typescale-label-small-font, var(--md-ref-typeface-plain, Roboto));font-size:var(--md-sys-typescale-label-small-size, 0.6875rem);font-weight:var(--md-sys-typescale-label-small-weight, var(--md-ref-typeface-weight-medium, 500));line-height:var(--md-sys-typescale-label-small-line-height, 1rem)}[name=supporting-text]{color:var(--md-sys-color-on-surface-variant, #49454f);font-family:var(--md-sys-typescale-body-medium-font, var(--md-ref-typeface-plain, Roboto));font-size:var(--md-sys-typescale-body-medium-size, 0.875rem);font-weight:var(--md-sys-typescale-body-medium-weight, var(--md-ref-typeface-weight-regular, 400));line-height:var(--md-sys-typescale-body-medium-line-height, 1.25rem)}[name=trailing-supporting-text]{color:var(--md-sys-color-on-surface-variant, #49454f);font-family:var(--md-sys-typescale-label-small-font, var(--md-ref-typeface-plain, Roboto));font-size:var(--md-sys-typescale-label-small-size, 0.6875rem);font-weight:var(--md-sys-typescale-label-small-weight, var(--md-ref-typeface-weight-medium, 500));line-height:var(--md-sys-typescale-label-small-line-height, 1rem)}[name=container]::slotted(*){inset:0;position:absolute}.default-slot{display:inline}.default-slot,.text ::slotted(*){overflow:hidden;text-overflow:ellipsis}.text{display:flex;flex:1;flex-direction:column;overflow:hidden}
`
/**
 * @license
 * Copyright 2023 Google LLC
 * SPDX-License-Identifier: Apache-2.0
 */;let Je=class extends Ye{};Je.styles=[Ze],Je=t([e("md-item")],Je);
/**
 * @license
 * Copyright 2023 Google LLC
 * SPDX-License-Identifier: Apache-2.0
 */
class Qe{constructor(t,e){this.host=t,this.internalTypeaheadText=null,this.onClick=()=>{this.host.keepOpen||this.host.dispatchEvent(ke(this.host,{kind:Ie}))},this.onKeydown=t=>{if(this.host.href&&"Enter"===t.code){const t=this.getInteractiveElement();t instanceof HTMLAnchorElement&&t.click()}if(t.defaultPrevented)return;const e=t.code;this.host.keepOpen&&"Escape"!==e||Pe(e)&&(t.preventDefault(),this.host.dispatchEvent(ke(this.host,{kind:Se,key:e})))},this.getHeadlineElements=e.getHeadlineElements,this.getSupportingTextElements=e.getSupportingTextElements,this.getDefaultElements=e.getDefaultElements,this.getInteractiveElement=e.getInteractiveElement,this.host.addController(this)}get typeaheadText(){if(null!==this.internalTypeaheadText)return this.internalTypeaheadText;const t=this.getHeadlineElements(),e=[];return t.forEach(t=>{t.textContent&&t.textContent.trim()&&e.push(t.textContent.trim())}),0===e.length&&this.getDefaultElements().forEach(t=>{t.textContent&&t.textContent.trim()&&e.push(t.textContent.trim())}),0===e.length&&this.getSupportingTextElements().forEach(t=>{t.textContent&&t.textContent.trim()&&e.push(t.textContent.trim())}),e.join(" ")}get tagName(){switch(this.host.type){case"link":return"a";case"button":return"button";default:return"li"}}get role(){return"option"===this.host.type?"option":"menuitem"}hostConnected(){this.host.toggleAttribute("md-menu-item",!0)}hostUpdate(){this.host.href&&(this.host.type="link")}setTypeaheadText(t){this.internalTypeaheadText=t}}
/**
 * @license
 * Copyright 2022 Google LLC
 * SPDX-License-Identifier: Apache-2.0
 */const ti=Jt(ft);class ei extends ti{constructor(){super(...arguments),this.disabled=!1,this.type="menuitem",this.href="",this.target="",this.keepOpen=!1,this.selected=!1,this.menuItemController=new Qe(this,{getHeadlineElements:()=>this.headlineElements,getSupportingTextElements:()=>this.supportingTextElements,getDefaultElements:()=>this.defaultElements,getInteractiveElement:()=>this.listItemRoot})}get typeaheadText(){return this.menuItemController.typeaheadText}set typeaheadText(t){this.menuItemController.setTypeaheadText(t)}render(){return this.renderListItem(J`
      <md-item>
        <div slot="container">
          ${this.renderRipple()} ${this.renderFocusRing()}
        </div>
        <slot name="start" slot="start"></slot>
        <slot name="end" slot="end"></slot>
        ${this.renderBody()}
      </md-item>
    `)}renderListItem(t){const e="link"===this.type;let i;switch(this.menuItemController.tagName){case"a":i=qt`a`;break;case"button":i=qt`button`;break;default:i=qt`li`}const o=e&&this.target?this.target:tt;return Vt`
      <${i}
        id="item"
        tabindex=${this.disabled&&!e?-1:0}
        role=${this.menuItemController.role}
        aria-label=${this.ariaLabel||tt}
        aria-selected=${this.ariaSelected||tt}
        aria-checked=${this.ariaChecked||tt}
        aria-expanded=${this.ariaExpanded||tt}
        aria-haspopup=${this.ariaHasPopup||tt}
        class="list-item ${Ot(this.getRenderClasses())}"
        href=${this.href||tt}
        target=${o}
        @click=${this.menuItemController.onClick}
        @keydown=${this.menuItemController.onKeydown}
      >${t}</${i}>
    `}renderRipple(){return J` <md-ripple
      part="ripple"
      for="item"
      ?disabled=${this.disabled}></md-ripple>`}renderFocusRing(){return J` <md-focus-ring
      part="focus-ring"
      for="item"
      inward></md-focus-ring>`}getRenderClasses(){return{disabled:this.disabled,selected:this.selected}}renderBody(){return J`
      <slot></slot>
      <slot name="overline" slot="overline"></slot>
      <slot name="headline" slot="headline"></slot>
      <slot name="supporting-text" slot="supporting-text"></slot>
      <slot
        name="trailing-supporting-text"
        slot="trailing-supporting-text"></slot>
    `}focus(){this.listItemRoot?.focus()}}ei.shadowRootOptions={...ft.shadowRootOptions,delegatesFocus:!0},t([A({type:Boolean,reflect:!0})],ei.prototype,"disabled",void 0),t([A()],ei.prototype,"type",void 0),t([A()],ei.prototype,"href",void 0),t([A()],ei.prototype,"target",void 0),t([A({type:Boolean,attribute:"keep-open"})],ei.prototype,"keepOpen",void 0),t([A({type:Boolean})],ei.prototype,"selected",void 0),t([S(".list-item")],ei.prototype,"listItemRoot",void 0),t([P({slot:"headline"})],ei.prototype,"headlineElements",void 0),t([P({slot:"supporting-text"})],ei.prototype,"supportingTextElements",void 0),t([function(t){return(e,i)=>{const{slot:o}=t??{},s="slot"+(o?`[name=${o}]`:":not([name])");return I(e,i,{get(){const e=this.renderRoot?.querySelector(s);return e?.assignedNodes(t)??[]}})}}({slot:""})],ei.prototype,"defaultElements",void 0),t([A({attribute:"typeahead-text"})],ei.prototype,"typeaheadText",null);
/**
 * @license
 * Copyright 2024 Google LLC
 * SPDX-License-Identifier: Apache-2.0
 */
const ii=n`:host{display:flex;--md-ripple-hover-color: var(--md-menu-item-hover-state-layer-color, var(--md-sys-color-on-surface, #1d1b20));--md-ripple-hover-opacity: var(--md-menu-item-hover-state-layer-opacity, 0.08);--md-ripple-pressed-color: var(--md-menu-item-pressed-state-layer-color, var(--md-sys-color-on-surface, #1d1b20));--md-ripple-pressed-opacity: var(--md-menu-item-pressed-state-layer-opacity, 0.12)}:host([disabled]){opacity:var(--md-menu-item-disabled-opacity, 0.3);pointer-events:none}md-focus-ring{z-index:1;--md-focus-ring-shape: 8px}a,button,li{background:none;border:none;padding:0;margin:0;text-align:unset;text-decoration:none}.list-item{border-radius:inherit;display:flex;flex:1;max-width:inherit;min-width:inherit;outline:none;-webkit-tap-highlight-color:rgba(0,0,0,0)}.list-item:not(.disabled){cursor:pointer}[slot=container]{pointer-events:none}md-ripple{border-radius:inherit}md-item{border-radius:inherit;flex:1;color:var(--md-menu-item-label-text-color, var(--md-sys-color-on-surface, #1d1b20));font-family:var(--md-menu-item-label-text-font, var(--md-sys-typescale-body-large-font, var(--md-ref-typeface-plain, Roboto)));font-size:var(--md-menu-item-label-text-size, var(--md-sys-typescale-body-large-size, 1rem));line-height:var(--md-menu-item-label-text-line-height, var(--md-sys-typescale-body-large-line-height, 1.5rem));font-weight:var(--md-menu-item-label-text-weight, var(--md-sys-typescale-body-large-weight, var(--md-ref-typeface-weight-regular, 400)));min-height:var(--md-menu-item-one-line-container-height, 56px);padding-top:var(--md-menu-item-top-space, 12px);padding-bottom:var(--md-menu-item-bottom-space, 12px);padding-inline-start:var(--md-menu-item-leading-space, 16px);padding-inline-end:var(--md-menu-item-trailing-space, 16px)}md-item[multiline]{min-height:var(--md-menu-item-two-line-container-height, 72px)}[slot=supporting-text]{color:var(--md-menu-item-supporting-text-color, var(--md-sys-color-on-surface-variant, #49454f));font-family:var(--md-menu-item-supporting-text-font, var(--md-sys-typescale-body-medium-font, var(--md-ref-typeface-plain, Roboto)));font-size:var(--md-menu-item-supporting-text-size, var(--md-sys-typescale-body-medium-size, 0.875rem));line-height:var(--md-menu-item-supporting-text-line-height, var(--md-sys-typescale-body-medium-line-height, 1.25rem));font-weight:var(--md-menu-item-supporting-text-weight, var(--md-sys-typescale-body-medium-weight, var(--md-ref-typeface-weight-regular, 400)))}[slot=trailing-supporting-text]{color:var(--md-menu-item-trailing-supporting-text-color, var(--md-sys-color-on-surface-variant, #49454f));font-family:var(--md-menu-item-trailing-supporting-text-font, var(--md-sys-typescale-label-small-font, var(--md-ref-typeface-plain, Roboto)));font-size:var(--md-menu-item-trailing-supporting-text-size, var(--md-sys-typescale-label-small-size, 0.6875rem));line-height:var(--md-menu-item-trailing-supporting-text-line-height, var(--md-sys-typescale-label-small-line-height, 1rem));font-weight:var(--md-menu-item-trailing-supporting-text-weight, var(--md-sys-typescale-label-small-weight, var(--md-ref-typeface-weight-medium, 500)))}:is([slot=start],[slot=end])::slotted(*){fill:currentColor}[slot=start]{color:var(--md-menu-item-leading-icon-color, var(--md-sys-color-on-surface-variant, #49454f))}[slot=end]{color:var(--md-menu-item-trailing-icon-color, var(--md-sys-color-on-surface-variant, #49454f))}.list-item{background-color:var(--md-menu-item-container-color, transparent)}.list-item.selected{background-color:var(--md-menu-item-selected-container-color, var(--md-sys-color-secondary-container, #e8def8))}.selected:not(.disabled) ::slotted(*){color:var(--md-menu-item-selected-label-text-color, var(--md-sys-color-on-secondary-container, #1d192b))}@media(forced-colors: active){:host([disabled]),:host([disabled]) slot{color:GrayText;opacity:1}.list-item{position:relative}.list-item.selected::before{content:"";position:absolute;inset:0;box-sizing:border-box;border-radius:inherit;pointer-events:none;border:3px double CanvasText}}
`
/**
 * @license
 * Copyright 2022 Google LLC
 * SPDX-License-Identifier: Apache-2.0
 */;let oi=class extends ei{};function si(t){return t.currentTarget===t.target&&(t.composedPath()[0]===t.target&&(!t.target.disabled&&!function(t){const e=ri;e&&(t.preventDefault(),t.stopImmediatePropagation());return async function(){ri=!0,await null,ri=!1}
/**
 * @license
 * Copyright 2019 Google LLC
 * SPDX-License-Identifier: Apache-2.0
 */(),e}(t)))}oi.styles=[ii],oi=t([e("md-menu-item")],oi);let ri=!1;const ai=Jt(oe(ft));class ni extends ai{get name(){return this.getAttribute("name")??""}set name(t){this.setAttribute("name",t)}get form(){return this[ee].form}constructor(){super(),this.disabled=!1,this.softDisabled=!1,this.href="",this.download="",this.target="",this.trailingIcon=!1,this.hasIcon=!1,this.type="submit",this.value="",this.addEventListener("click",this.handleClick.bind(this))}focus(){this.buttonElement?.focus()}blur(){this.buttonElement?.blur()}render(){const t=this.disabled||this.softDisabled,e=this.href?this.renderLink():this.renderButton(),i=this.href?"link":"button";return J`
      ${this.renderElevationOrOutline?.()}
      <div class="background"></div>
      <md-focus-ring part="focus-ring" for=${i}></md-focus-ring>
      <md-ripple
        part="ripple"
        for=${i}
        ?disabled="${t}"></md-ripple>
      ${e}
    `}renderButton(){const{ariaLabel:t,ariaHasPopup:e,ariaExpanded:i}=this;return J`<button
      id="button"
      class="button"
      ?disabled=${this.disabled}
      aria-disabled=${this.softDisabled||tt}
      aria-label="${t||tt}"
      aria-haspopup="${e||tt}"
      aria-expanded="${i||tt}">
      ${this.renderContent()}
    </button>`}renderLink(){const{ariaLabel:t,ariaHasPopup:e,ariaExpanded:i}=this;return J`<a
      id="link"
      class="button"
      aria-label="${t||tt}"
      aria-haspopup="${e||tt}"
      aria-expanded="${i||tt}"
      aria-disabled=${this.disabled||this.softDisabled||tt}
      tabindex="${this.disabled&&!this.softDisabled?-1:tt}"
      href=${this.href}
      download=${this.download||tt}
      target=${this.target||tt}
      >${this.renderContent()}
    </a>`}renderContent(){const t=J`<slot
      name="icon"
      @slotchange="${this.handleSlotChange}"></slot>`;return J`
      <span class="touch"></span>
      ${this.trailingIcon?tt:t}
      <span class="label"><slot></slot></span>
      ${this.trailingIcon?t:tt}
    `}handleClick(t){if(this.softDisabled||this.disabled&&this.href)return t.stopImmediatePropagation(),void t.preventDefault();si(t)&&this.buttonElement&&(this.focus(),
/**
 * @license
 * Copyright 2021 Google LLC
 * SPDX-License-Identifier: Apache-2.0
 */
function(t){const e=new MouseEvent("click",{bubbles:!0});t.dispatchEvent(e)}(this.buttonElement))}handleSlotChange(){this.hasIcon=this.assignedIcons.length>0}}se(ni),ni.formAssociated=!0,ni.shadowRootOptions={mode:"open",delegatesFocus:!0},t([A({type:Boolean,reflect:!0})],ni.prototype,"disabled",void 0),t([A({type:Boolean,attribute:"soft-disabled",reflect:!0})],ni.prototype,"softDisabled",void 0),t([A()],ni.prototype,"href",void 0),t([A()],ni.prototype,"download",void 0),t([A()],ni.prototype,"target",void 0),t([A({type:Boolean,attribute:"trailing-icon",reflect:!0})],ni.prototype,"trailingIcon",void 0),t([A({type:Boolean,attribute:"has-icon",reflect:!0})],ni.prototype,"hasIcon",void 0),t([A()],ni.prototype,"type",void 0),t([A({reflect:!0})],ni.prototype,"value",void 0),t([S(".button")],ni.prototype,"buttonElement",void 0),t([P({slot:"icon",flatten:!0})],ni.prototype,"assignedIcons",void 0);
/**
 * @license
 * Copyright 2021 Google LLC
 * SPDX-License-Identifier: Apache-2.0
 */
class li extends ni{renderElevationOrOutline(){return J`<md-elevation part="elevation"></md-elevation>`}}
/**
 * @license
 * Copyright 2024 Google LLC
 * SPDX-License-Identifier: Apache-2.0
 */const di=n`:host{--_container-color: var(--md-filled-button-container-color, var(--md-sys-color-primary, #6750a4));--_container-elevation: var(--md-filled-button-container-elevation, 0);--_container-height: var(--md-filled-button-container-height, 40px);--_container-shadow-color: var(--md-filled-button-container-shadow-color, var(--md-sys-color-shadow, #000));--_disabled-container-color: var(--md-filled-button-disabled-container-color, var(--md-sys-color-on-surface, #1d1b20));--_disabled-container-elevation: var(--md-filled-button-disabled-container-elevation, 0);--_disabled-container-opacity: var(--md-filled-button-disabled-container-opacity, 0.12);--_disabled-label-text-color: var(--md-filled-button-disabled-label-text-color, var(--md-sys-color-on-surface, #1d1b20));--_disabled-label-text-opacity: var(--md-filled-button-disabled-label-text-opacity, 0.38);--_focus-container-elevation: var(--md-filled-button-focus-container-elevation, 0);--_focus-label-text-color: var(--md-filled-button-focus-label-text-color, var(--md-sys-color-on-primary, #fff));--_hover-container-elevation: var(--md-filled-button-hover-container-elevation, 1);--_hover-label-text-color: var(--md-filled-button-hover-label-text-color, var(--md-sys-color-on-primary, #fff));--_hover-state-layer-color: var(--md-filled-button-hover-state-layer-color, var(--md-sys-color-on-primary, #fff));--_hover-state-layer-opacity: var(--md-filled-button-hover-state-layer-opacity, 0.08);--_label-text-color: var(--md-filled-button-label-text-color, var(--md-sys-color-on-primary, #fff));--_label-text-font: var(--md-filled-button-label-text-font, var(--md-sys-typescale-label-large-font, var(--md-ref-typeface-plain, Roboto)));--_label-text-line-height: var(--md-filled-button-label-text-line-height, var(--md-sys-typescale-label-large-line-height, 1.25rem));--_label-text-size: var(--md-filled-button-label-text-size, var(--md-sys-typescale-label-large-size, 0.875rem));--_label-text-weight: var(--md-filled-button-label-text-weight, var(--md-sys-typescale-label-large-weight, var(--md-ref-typeface-weight-medium, 500)));--_pressed-container-elevation: var(--md-filled-button-pressed-container-elevation, 0);--_pressed-label-text-color: var(--md-filled-button-pressed-label-text-color, var(--md-sys-color-on-primary, #fff));--_pressed-state-layer-color: var(--md-filled-button-pressed-state-layer-color, var(--md-sys-color-on-primary, #fff));--_pressed-state-layer-opacity: var(--md-filled-button-pressed-state-layer-opacity, 0.12);--_disabled-icon-color: var(--md-filled-button-disabled-icon-color, var(--md-sys-color-on-surface, #1d1b20));--_disabled-icon-opacity: var(--md-filled-button-disabled-icon-opacity, 0.38);--_focus-icon-color: var(--md-filled-button-focus-icon-color, var(--md-sys-color-on-primary, #fff));--_hover-icon-color: var(--md-filled-button-hover-icon-color, var(--md-sys-color-on-primary, #fff));--_icon-color: var(--md-filled-button-icon-color, var(--md-sys-color-on-primary, #fff));--_icon-size: var(--md-filled-button-icon-size, 18px);--_pressed-icon-color: var(--md-filled-button-pressed-icon-color, var(--md-sys-color-on-primary, #fff));--_container-shape-start-start: var(--md-filled-button-container-shape-start-start, var(--md-filled-button-container-shape, var(--md-sys-shape-corner-full, 9999px)));--_container-shape-start-end: var(--md-filled-button-container-shape-start-end, var(--md-filled-button-container-shape, var(--md-sys-shape-corner-full, 9999px)));--_container-shape-end-end: var(--md-filled-button-container-shape-end-end, var(--md-filled-button-container-shape, var(--md-sys-shape-corner-full, 9999px)));--_container-shape-end-start: var(--md-filled-button-container-shape-end-start, var(--md-filled-button-container-shape, var(--md-sys-shape-corner-full, 9999px)));--_leading-space: var(--md-filled-button-leading-space, 24px);--_trailing-space: var(--md-filled-button-trailing-space, 24px);--_with-leading-icon-leading-space: var(--md-filled-button-with-leading-icon-leading-space, 16px);--_with-leading-icon-trailing-space: var(--md-filled-button-with-leading-icon-trailing-space, 24px);--_with-trailing-icon-leading-space: var(--md-filled-button-with-trailing-icon-leading-space, 24px);--_with-trailing-icon-trailing-space: var(--md-filled-button-with-trailing-icon-trailing-space, 16px)}
`,ci=n`md-elevation{transition-duration:280ms}:host(:is([disabled],[soft-disabled])) md-elevation{transition:none}md-elevation{--md-elevation-level: var(--_container-elevation);--md-elevation-shadow-color: var(--_container-shadow-color)}:host(:focus-within) md-elevation{--md-elevation-level: var(--_focus-container-elevation)}:host(:hover) md-elevation{--md-elevation-level: var(--_hover-container-elevation)}:host(:active) md-elevation{--md-elevation-level: var(--_pressed-container-elevation)}:host(:is([disabled],[soft-disabled])) md-elevation{--md-elevation-level: var(--_disabled-container-elevation)}
`,hi=n`:host{border-start-start-radius:var(--_container-shape-start-start);border-start-end-radius:var(--_container-shape-start-end);border-end-start-radius:var(--_container-shape-end-start);border-end-end-radius:var(--_container-shape-end-end);box-sizing:border-box;cursor:pointer;display:inline-flex;gap:8px;min-height:var(--_container-height);outline:none;padding-block:calc((var(--_container-height) - max(var(--_label-text-line-height),var(--_icon-size)))/2);padding-inline-start:var(--_leading-space);padding-inline-end:var(--_trailing-space);place-content:center;place-items:center;position:relative;font-family:var(--_label-text-font);font-size:var(--_label-text-size);line-height:var(--_label-text-line-height);font-weight:var(--_label-text-weight);text-overflow:ellipsis;text-wrap:nowrap;user-select:none;-webkit-tap-highlight-color:rgba(0,0,0,0);vertical-align:top;--md-ripple-hover-color: var(--_hover-state-layer-color);--md-ripple-pressed-color: var(--_pressed-state-layer-color);--md-ripple-hover-opacity: var(--_hover-state-layer-opacity);--md-ripple-pressed-opacity: var(--_pressed-state-layer-opacity)}md-focus-ring{--md-focus-ring-shape-start-start: var(--_container-shape-start-start);--md-focus-ring-shape-start-end: var(--_container-shape-start-end);--md-focus-ring-shape-end-end: var(--_container-shape-end-end);--md-focus-ring-shape-end-start: var(--_container-shape-end-start)}:host(:is([disabled],[soft-disabled])){cursor:default;pointer-events:none}.button{border-radius:inherit;cursor:inherit;display:inline-flex;align-items:center;justify-content:center;border:none;outline:none;-webkit-appearance:none;vertical-align:middle;background:rgba(0,0,0,0);text-decoration:none;min-width:calc(64px - var(--_leading-space) - var(--_trailing-space));width:100%;z-index:0;height:100%;font:inherit;color:var(--_label-text-color);padding:0;gap:inherit;text-transform:inherit}.button::-moz-focus-inner{padding:0;border:0}:host(:hover) .button{color:var(--_hover-label-text-color)}:host(:focus-within) .button{color:var(--_focus-label-text-color)}:host(:active) .button{color:var(--_pressed-label-text-color)}.background{background:var(--_container-color);border-radius:inherit;inset:0;position:absolute}.label{overflow:hidden}:is(.button,.label,.label slot),.label ::slotted(*){text-overflow:inherit}:host(:is([disabled],[soft-disabled])) .label{color:var(--_disabled-label-text-color);opacity:var(--_disabled-label-text-opacity)}:host(:is([disabled],[soft-disabled])) .background{background:var(--_disabled-container-color);opacity:var(--_disabled-container-opacity)}@media(forced-colors: active){.background{border:1px solid CanvasText}:host(:is([disabled],[soft-disabled])){--_disabled-icon-color: GrayText;--_disabled-icon-opacity: 1;--_disabled-container-opacity: 1;--_disabled-label-text-color: GrayText;--_disabled-label-text-opacity: 1}}:host([has-icon]:not([trailing-icon])){padding-inline-start:var(--_with-leading-icon-leading-space);padding-inline-end:var(--_with-leading-icon-trailing-space)}:host([has-icon][trailing-icon]){padding-inline-start:var(--_with-trailing-icon-leading-space);padding-inline-end:var(--_with-trailing-icon-trailing-space)}::slotted([slot=icon]){display:inline-flex;position:relative;writing-mode:horizontal-tb;fill:currentColor;flex-shrink:0;color:var(--_icon-color);font-size:var(--_icon-size);inline-size:var(--_icon-size);block-size:var(--_icon-size)}:host(:hover) ::slotted([slot=icon]){color:var(--_hover-icon-color)}:host(:focus-within) ::slotted([slot=icon]){color:var(--_focus-icon-color)}:host(:active) ::slotted([slot=icon]){color:var(--_pressed-icon-color)}:host(:is([disabled],[soft-disabled])) ::slotted([slot=icon]){color:var(--_disabled-icon-color);opacity:var(--_disabled-icon-opacity)}.touch{position:absolute;top:50%;height:48px;left:0;right:0;transform:translateY(-50%)}:host([touch-target=wrapper]){margin:max(0px,(48px - var(--_container-height))/2) 0}:host([touch-target=none]) .touch{display:none}
`
/**
 * @license
 * Copyright 2021 Google LLC
 * SPDX-License-Identifier: Apache-2.0
 */;
/**
 * @license
 * Copyright 2024 Google LLC
 * SPDX-License-Identifier: Apache-2.0
 */let pi=class extends li{};pi.styles=[hi,ci,di],pi=t([e("md-filled-button")],pi);
/**
 * @license
 * Copyright 2021 Google LLC
 * SPDX-License-Identifier: Apache-2.0
 */
class ui extends ni{renderElevationOrOutline(){return J`<md-elevation part="elevation"></md-elevation>`}}
/**
 * @license
 * Copyright 2024 Google LLC
 * SPDX-License-Identifier: Apache-2.0
 */const mi=n`:host{--_container-color: var(--md-filled-tonal-button-container-color, var(--md-sys-color-secondary-container, #e8def8));--_container-elevation: var(--md-filled-tonal-button-container-elevation, 0);--_container-height: var(--md-filled-tonal-button-container-height, 40px);--_container-shadow-color: var(--md-filled-tonal-button-container-shadow-color, var(--md-sys-color-shadow, #000));--_disabled-container-color: var(--md-filled-tonal-button-disabled-container-color, var(--md-sys-color-on-surface, #1d1b20));--_disabled-container-elevation: var(--md-filled-tonal-button-disabled-container-elevation, 0);--_disabled-container-opacity: var(--md-filled-tonal-button-disabled-container-opacity, 0.12);--_disabled-label-text-color: var(--md-filled-tonal-button-disabled-label-text-color, var(--md-sys-color-on-surface, #1d1b20));--_disabled-label-text-opacity: var(--md-filled-tonal-button-disabled-label-text-opacity, 0.38);--_focus-container-elevation: var(--md-filled-tonal-button-focus-container-elevation, 0);--_focus-label-text-color: var(--md-filled-tonal-button-focus-label-text-color, var(--md-sys-color-on-secondary-container, #1d192b));--_hover-container-elevation: var(--md-filled-tonal-button-hover-container-elevation, 1);--_hover-label-text-color: var(--md-filled-tonal-button-hover-label-text-color, var(--md-sys-color-on-secondary-container, #1d192b));--_hover-state-layer-color: var(--md-filled-tonal-button-hover-state-layer-color, var(--md-sys-color-on-secondary-container, #1d192b));--_hover-state-layer-opacity: var(--md-filled-tonal-button-hover-state-layer-opacity, 0.08);--_label-text-color: var(--md-filled-tonal-button-label-text-color, var(--md-sys-color-on-secondary-container, #1d192b));--_label-text-font: var(--md-filled-tonal-button-label-text-font, var(--md-sys-typescale-label-large-font, var(--md-ref-typeface-plain, Roboto)));--_label-text-line-height: var(--md-filled-tonal-button-label-text-line-height, var(--md-sys-typescale-label-large-line-height, 1.25rem));--_label-text-size: var(--md-filled-tonal-button-label-text-size, var(--md-sys-typescale-label-large-size, 0.875rem));--_label-text-weight: var(--md-filled-tonal-button-label-text-weight, var(--md-sys-typescale-label-large-weight, var(--md-ref-typeface-weight-medium, 500)));--_pressed-container-elevation: var(--md-filled-tonal-button-pressed-container-elevation, 0);--_pressed-label-text-color: var(--md-filled-tonal-button-pressed-label-text-color, var(--md-sys-color-on-secondary-container, #1d192b));--_pressed-state-layer-color: var(--md-filled-tonal-button-pressed-state-layer-color, var(--md-sys-color-on-secondary-container, #1d192b));--_pressed-state-layer-opacity: var(--md-filled-tonal-button-pressed-state-layer-opacity, 0.12);--_disabled-icon-color: var(--md-filled-tonal-button-disabled-icon-color, var(--md-sys-color-on-surface, #1d1b20));--_disabled-icon-opacity: var(--md-filled-tonal-button-disabled-icon-opacity, 0.38);--_focus-icon-color: var(--md-filled-tonal-button-focus-icon-color, var(--md-sys-color-on-secondary-container, #1d192b));--_hover-icon-color: var(--md-filled-tonal-button-hover-icon-color, var(--md-sys-color-on-secondary-container, #1d192b));--_icon-color: var(--md-filled-tonal-button-icon-color, var(--md-sys-color-on-secondary-container, #1d192b));--_icon-size: var(--md-filled-tonal-button-icon-size, 18px);--_pressed-icon-color: var(--md-filled-tonal-button-pressed-icon-color, var(--md-sys-color-on-secondary-container, #1d192b));--_container-shape-start-start: var(--md-filled-tonal-button-container-shape-start-start, var(--md-filled-tonal-button-container-shape, var(--md-sys-shape-corner-full, 9999px)));--_container-shape-start-end: var(--md-filled-tonal-button-container-shape-start-end, var(--md-filled-tonal-button-container-shape, var(--md-sys-shape-corner-full, 9999px)));--_container-shape-end-end: var(--md-filled-tonal-button-container-shape-end-end, var(--md-filled-tonal-button-container-shape, var(--md-sys-shape-corner-full, 9999px)));--_container-shape-end-start: var(--md-filled-tonal-button-container-shape-end-start, var(--md-filled-tonal-button-container-shape, var(--md-sys-shape-corner-full, 9999px)));--_leading-space: var(--md-filled-tonal-button-leading-space, 24px);--_trailing-space: var(--md-filled-tonal-button-trailing-space, 24px);--_with-leading-icon-leading-space: var(--md-filled-tonal-button-with-leading-icon-leading-space, 16px);--_with-leading-icon-trailing-space: var(--md-filled-tonal-button-with-leading-icon-trailing-space, 24px);--_with-trailing-icon-leading-space: var(--md-filled-tonal-button-with-trailing-icon-leading-space, 24px);--_with-trailing-icon-trailing-space: var(--md-filled-tonal-button-with-trailing-icon-trailing-space, 16px)}
`
/**
 * @license
 * Copyright 2021 Google LLC
 * SPDX-License-Identifier: Apache-2.0
 */;let vi=class extends ui{};vi.styles=[hi,ci,mi],vi=t([e("md-filled-tonal-button")],vi);
/**
 * @license
 * Copyright 2021 Google LLC
 * SPDX-License-Identifier: Apache-2.0
 */
class fi extends ni{renderElevationOrOutline(){return J`<div class="outline"></div>`}}
/**
 * @license
 * Copyright 2024 Google LLC
 * SPDX-License-Identifier: Apache-2.0
 */const bi=n`:host{--_container-height: var(--md-outlined-button-container-height, 40px);--_disabled-label-text-color: var(--md-outlined-button-disabled-label-text-color, var(--md-sys-color-on-surface, #1d1b20));--_disabled-label-text-opacity: var(--md-outlined-button-disabled-label-text-opacity, 0.38);--_disabled-outline-color: var(--md-outlined-button-disabled-outline-color, var(--md-sys-color-on-surface, #1d1b20));--_disabled-outline-opacity: var(--md-outlined-button-disabled-outline-opacity, 0.12);--_focus-label-text-color: var(--md-outlined-button-focus-label-text-color, var(--md-sys-color-primary, #6750a4));--_hover-label-text-color: var(--md-outlined-button-hover-label-text-color, var(--md-sys-color-primary, #6750a4));--_hover-state-layer-color: var(--md-outlined-button-hover-state-layer-color, var(--md-sys-color-primary, #6750a4));--_hover-state-layer-opacity: var(--md-outlined-button-hover-state-layer-opacity, 0.08);--_label-text-color: var(--md-outlined-button-label-text-color, var(--md-sys-color-primary, #6750a4));--_label-text-font: var(--md-outlined-button-label-text-font, var(--md-sys-typescale-label-large-font, var(--md-ref-typeface-plain, Roboto)));--_label-text-line-height: var(--md-outlined-button-label-text-line-height, var(--md-sys-typescale-label-large-line-height, 1.25rem));--_label-text-size: var(--md-outlined-button-label-text-size, var(--md-sys-typescale-label-large-size, 0.875rem));--_label-text-weight: var(--md-outlined-button-label-text-weight, var(--md-sys-typescale-label-large-weight, var(--md-ref-typeface-weight-medium, 500)));--_outline-color: var(--md-outlined-button-outline-color, var(--md-sys-color-outline, #79747e));--_outline-width: var(--md-outlined-button-outline-width, 1px);--_pressed-label-text-color: var(--md-outlined-button-pressed-label-text-color, var(--md-sys-color-primary, #6750a4));--_pressed-outline-color: var(--md-outlined-button-pressed-outline-color, var(--md-sys-color-outline, #79747e));--_pressed-state-layer-color: var(--md-outlined-button-pressed-state-layer-color, var(--md-sys-color-primary, #6750a4));--_pressed-state-layer-opacity: var(--md-outlined-button-pressed-state-layer-opacity, 0.12);--_disabled-icon-color: var(--md-outlined-button-disabled-icon-color, var(--md-sys-color-on-surface, #1d1b20));--_disabled-icon-opacity: var(--md-outlined-button-disabled-icon-opacity, 0.38);--_focus-icon-color: var(--md-outlined-button-focus-icon-color, var(--md-sys-color-primary, #6750a4));--_hover-icon-color: var(--md-outlined-button-hover-icon-color, var(--md-sys-color-primary, #6750a4));--_icon-color: var(--md-outlined-button-icon-color, var(--md-sys-color-primary, #6750a4));--_icon-size: var(--md-outlined-button-icon-size, 18px);--_pressed-icon-color: var(--md-outlined-button-pressed-icon-color, var(--md-sys-color-primary, #6750a4));--_container-shape-start-start: var(--md-outlined-button-container-shape-start-start, var(--md-outlined-button-container-shape, var(--md-sys-shape-corner-full, 9999px)));--_container-shape-start-end: var(--md-outlined-button-container-shape-start-end, var(--md-outlined-button-container-shape, var(--md-sys-shape-corner-full, 9999px)));--_container-shape-end-end: var(--md-outlined-button-container-shape-end-end, var(--md-outlined-button-container-shape, var(--md-sys-shape-corner-full, 9999px)));--_container-shape-end-start: var(--md-outlined-button-container-shape-end-start, var(--md-outlined-button-container-shape, var(--md-sys-shape-corner-full, 9999px)));--_leading-space: var(--md-outlined-button-leading-space, 24px);--_trailing-space: var(--md-outlined-button-trailing-space, 24px);--_with-leading-icon-leading-space: var(--md-outlined-button-with-leading-icon-leading-space, 16px);--_with-leading-icon-trailing-space: var(--md-outlined-button-with-leading-icon-trailing-space, 24px);--_with-trailing-icon-leading-space: var(--md-outlined-button-with-trailing-icon-leading-space, 24px);--_with-trailing-icon-trailing-space: var(--md-outlined-button-with-trailing-icon-trailing-space, 16px);--_container-color: none;--_disabled-container-color: none;--_disabled-container-opacity: 0}.outline{inset:0;border-style:solid;position:absolute;box-sizing:border-box;border-color:var(--_outline-color);border-start-start-radius:var(--_container-shape-start-start);border-start-end-radius:var(--_container-shape-start-end);border-end-start-radius:var(--_container-shape-end-start);border-end-end-radius:var(--_container-shape-end-end)}:host(:active) .outline{border-color:var(--_pressed-outline-color)}:host(:is([disabled],[soft-disabled])) .outline{border-color:var(--_disabled-outline-color);opacity:var(--_disabled-outline-opacity)}@media(forced-colors: active){:host(:is([disabled],[soft-disabled])) .background{border-color:GrayText}:host(:is([disabled],[soft-disabled])) .outline{opacity:1}}.outline,md-ripple{border-width:var(--_outline-width)}md-ripple{inline-size:calc(100% - 2*var(--_outline-width));block-size:calc(100% - 2*var(--_outline-width));border-style:solid;border-color:rgba(0,0,0,0)}
`
/**
 * @license
 * Copyright 2021 Google LLC
 * SPDX-License-Identifier: Apache-2.0
 */;let yi=class extends fi{};yi.styles=[hi,bi],yi=t([e("md-outlined-button")],yi);
/**
 * @license
 * Copyright 2021 Google LLC
 * SPDX-License-Identifier: Apache-2.0
 */
class gi extends ni{}
/**
 * @license
 * Copyright 2024 Google LLC
 * SPDX-License-Identifier: Apache-2.0
 */const xi=n`:host{--_container-height: var(--md-text-button-container-height, 40px);--_disabled-label-text-color: var(--md-text-button-disabled-label-text-color, var(--md-sys-color-on-surface, #1d1b20));--_disabled-label-text-opacity: var(--md-text-button-disabled-label-text-opacity, 0.38);--_focus-label-text-color: var(--md-text-button-focus-label-text-color, var(--md-sys-color-primary, #6750a4));--_hover-label-text-color: var(--md-text-button-hover-label-text-color, var(--md-sys-color-primary, #6750a4));--_hover-state-layer-color: var(--md-text-button-hover-state-layer-color, var(--md-sys-color-primary, #6750a4));--_hover-state-layer-opacity: var(--md-text-button-hover-state-layer-opacity, 0.08);--_label-text-color: var(--md-text-button-label-text-color, var(--md-sys-color-primary, #6750a4));--_label-text-font: var(--md-text-button-label-text-font, var(--md-sys-typescale-label-large-font, var(--md-ref-typeface-plain, Roboto)));--_label-text-line-height: var(--md-text-button-label-text-line-height, var(--md-sys-typescale-label-large-line-height, 1.25rem));--_label-text-size: var(--md-text-button-label-text-size, var(--md-sys-typescale-label-large-size, 0.875rem));--_label-text-weight: var(--md-text-button-label-text-weight, var(--md-sys-typescale-label-large-weight, var(--md-ref-typeface-weight-medium, 500)));--_pressed-label-text-color: var(--md-text-button-pressed-label-text-color, var(--md-sys-color-primary, #6750a4));--_pressed-state-layer-color: var(--md-text-button-pressed-state-layer-color, var(--md-sys-color-primary, #6750a4));--_pressed-state-layer-opacity: var(--md-text-button-pressed-state-layer-opacity, 0.12);--_disabled-icon-color: var(--md-text-button-disabled-icon-color, var(--md-sys-color-on-surface, #1d1b20));--_disabled-icon-opacity: var(--md-text-button-disabled-icon-opacity, 0.38);--_focus-icon-color: var(--md-text-button-focus-icon-color, var(--md-sys-color-primary, #6750a4));--_hover-icon-color: var(--md-text-button-hover-icon-color, var(--md-sys-color-primary, #6750a4));--_icon-color: var(--md-text-button-icon-color, var(--md-sys-color-primary, #6750a4));--_icon-size: var(--md-text-button-icon-size, 18px);--_pressed-icon-color: var(--md-text-button-pressed-icon-color, var(--md-sys-color-primary, #6750a4));--_container-shape-start-start: var(--md-text-button-container-shape-start-start, var(--md-text-button-container-shape, var(--md-sys-shape-corner-full, 9999px)));--_container-shape-start-end: var(--md-text-button-container-shape-start-end, var(--md-text-button-container-shape, var(--md-sys-shape-corner-full, 9999px)));--_container-shape-end-end: var(--md-text-button-container-shape-end-end, var(--md-text-button-container-shape, var(--md-sys-shape-corner-full, 9999px)));--_container-shape-end-start: var(--md-text-button-container-shape-end-start, var(--md-text-button-container-shape, var(--md-sys-shape-corner-full, 9999px)));--_leading-space: var(--md-text-button-leading-space, 12px);--_trailing-space: var(--md-text-button-trailing-space, 12px);--_with-leading-icon-leading-space: var(--md-text-button-with-leading-icon-leading-space, 12px);--_with-leading-icon-trailing-space: var(--md-text-button-with-leading-icon-trailing-space, 16px);--_with-trailing-icon-leading-space: var(--md-text-button-with-trailing-icon-leading-space, 16px);--_with-trailing-icon-trailing-space: var(--md-text-button-with-trailing-icon-trailing-space, 12px);--_container-color: none;--_disabled-container-color: none;--_disabled-container-opacity: 0}
`
/**
 * @license
 * Copyright 2021 Google LLC
 * SPDX-License-Identifier: Apache-2.0
 */;let _i=class extends gi{};_i.styles=[hi,xi],_i=t([e("md-text-button")],_i);
/**
 * @license
 * Copyright 2021 Google LLC
 * SPDX-License-Identifier: Apache-2.0
 */
const wi=new Set(Object.values(Ee));class $i extends ft{get items(){return this.listController.items}constructor(){super(),this.listController=new Ae({isItem:t=>t.hasAttribute("md-list-item"),getPossibleItems:()=>this.slotItems,isRtl:()=>"rtl"===getComputedStyle(this).direction,deactivateItem:t=>{t.tabIndex=-1},activateItem:t=>{t.tabIndex=0},isNavigableKey:t=>wi.has(t),isActivatable:t=>!t.disabled&&"text"!==t.type}),this.internals=this.attachInternals(),this.internals.role="list",this.addEventListener("keydown",this.listController.handleKeydown)}render(){return J`
      <slot
        @deactivate-items=${this.listController.onDeactivateItems}
        @request-activation=${this.listController.onRequestActivation}
        @slotchange=${this.listController.onSlotchange}>
      </slot>
    `}activateNextItem(){return this.listController.activateNextItem()}activatePreviousItem(){return this.listController.activatePreviousItem()}}t([P({flatten:!0})],$i.prototype,"slotItems",void 0);
/**
 * @license
 * Copyright 2024 Google LLC
 * SPDX-License-Identifier: Apache-2.0
 */
const Ci=n`:host{background:var(--md-list-container-color, var(--md-sys-color-surface, #fef7ff));color:unset;display:flex;flex-direction:column;outline:none;padding:8px 0;position:relative}
`
/**
 * @license
 * Copyright 2021 Google LLC
 * SPDX-License-Identifier: Apache-2.0
 */;let Ei=class extends $i{};Ei.styles=[Ci],Ei=t([e("md-list")],Ei);
/**
 * @license
 * Copyright 2022 Google LLC
 * SPDX-License-Identifier: Apache-2.0
 */
const Ai=Jt(ft);class ki extends Ai{constructor(){super(...arguments),this.disabled=!1,this.type="text",this.isListItem=!0,this.href="",this.target=""}get isDisabled(){return this.disabled&&"link"!==this.type}willUpdate(t){this.href&&(this.type="link"),super.willUpdate(t)}render(){return this.renderListItem(J`
      <md-item>
        <div slot="container">
          ${this.renderRipple()} ${this.renderFocusRing()}
        </div>
        <slot name="start" slot="start"></slot>
        <slot name="end" slot="end"></slot>
        ${this.renderBody()}
      </md-item>
    `)}renderListItem(t){const e="link"===this.type;let i;switch(this.type){case"link":i=qt`a`;break;case"button":i=qt`button`;break;default:i=qt`li`}const o="text"!==this.type,s=e&&this.target?this.target:tt;return Vt`
      <${i}
        id="item"
        tabindex="${this.isDisabled||!o?-1:0}"
        ?disabled=${this.isDisabled}
        role="listitem"
        aria-selected=${this.ariaSelected||tt}
        aria-checked=${this.ariaChecked||tt}
        aria-expanded=${this.ariaExpanded||tt}
        aria-haspopup=${this.ariaHasPopup||tt}
        class="list-item ${Ot(this.getRenderClasses())}"
        href=${this.href||tt}
        target=${s}
        @focus=${this.onFocus}
      >${t}</${i}>
    `}renderRipple(){return"text"===this.type?tt:J` <md-ripple
      part="ripple"
      for="item"
      ?disabled=${this.isDisabled}></md-ripple>`}renderFocusRing(){return"text"===this.type?tt:J` <md-focus-ring
      @visibility-changed=${this.onFocusRingVisibilityChanged}
      part="focus-ring"
      for="item"
      inward></md-focus-ring>`}onFocusRingVisibilityChanged(t){}getRenderClasses(){return{disabled:this.isDisabled}}renderBody(){return J`
      <slot></slot>
      <slot name="overline" slot="overline"></slot>
      <slot name="headline" slot="headline"></slot>
      <slot name="supporting-text" slot="supporting-text"></slot>
      <slot
        name="trailing-supporting-text"
        slot="trailing-supporting-text"></slot>
    `}onFocus(){-1===this.tabIndex&&this.dispatchEvent(new Event("request-activation",{bubbles:!0,composed:!0}))}focus(){this.listItemRoot?.focus()}click(){this.listItemRoot?this.listItemRoot.click():super.click()}}ki.shadowRootOptions={...ft.shadowRootOptions,delegatesFocus:!0},t([A({type:Boolean,reflect:!0})],ki.prototype,"disabled",void 0),t([A({reflect:!0})],ki.prototype,"type",void 0),t([A({type:Boolean,attribute:"md-list-item",reflect:!0})],ki.prototype,"isListItem",void 0),t([A()],ki.prototype,"href",void 0),t([A()],ki.prototype,"target",void 0),t([S(".list-item")],ki.prototype,"listItemRoot",void 0);
/**
 * @license
 * Copyright 2024 Google LLC
 * SPDX-License-Identifier: Apache-2.0
 */
const Ii=n`:host{display:flex;-webkit-tap-highlight-color:rgba(0,0,0,0);--md-ripple-hover-color: var(--md-list-item-hover-state-layer-color, var(--md-sys-color-on-surface, #1d1b20));--md-ripple-hover-opacity: var(--md-list-item-hover-state-layer-opacity, 0.08);--md-ripple-pressed-color: var(--md-list-item-pressed-state-layer-color, var(--md-sys-color-on-surface, #1d1b20));--md-ripple-pressed-opacity: var(--md-list-item-pressed-state-layer-opacity, 0.12)}:host(:is([type=button]:not([disabled]),[type=link])){cursor:pointer}md-focus-ring{z-index:1;--md-focus-ring-shape: 8px}a,button,li{background:none;border:none;cursor:inherit;padding:0;margin:0;text-align:unset;text-decoration:none}.list-item{border-radius:inherit;display:flex;flex:1;max-width:inherit;min-width:inherit;outline:none;-webkit-tap-highlight-color:rgba(0,0,0,0);width:100%}.list-item.interactive{cursor:pointer}.list-item.disabled{opacity:var(--md-list-item-disabled-opacity, 0.3);pointer-events:none}[slot=container]{pointer-events:none}md-ripple{border-radius:inherit}md-item{border-radius:inherit;flex:1;height:100%;color:var(--md-list-item-label-text-color, var(--md-sys-color-on-surface, #1d1b20));font-family:var(--md-list-item-label-text-font, var(--md-sys-typescale-body-large-font, var(--md-ref-typeface-plain, Roboto)));font-size:var(--md-list-item-label-text-size, var(--md-sys-typescale-body-large-size, 1rem));line-height:var(--md-list-item-label-text-line-height, var(--md-sys-typescale-body-large-line-height, 1.5rem));font-weight:var(--md-list-item-label-text-weight, var(--md-sys-typescale-body-large-weight, var(--md-ref-typeface-weight-regular, 400)));min-height:var(--md-list-item-one-line-container-height, 56px);padding-top:var(--md-list-item-top-space, 12px);padding-bottom:var(--md-list-item-bottom-space, 12px);padding-inline-start:var(--md-list-item-leading-space, 16px);padding-inline-end:var(--md-list-item-trailing-space, 16px)}md-item[multiline]{min-height:var(--md-list-item-two-line-container-height, 72px)}[slot=supporting-text]{color:var(--md-list-item-supporting-text-color, var(--md-sys-color-on-surface-variant, #49454f));font-family:var(--md-list-item-supporting-text-font, var(--md-sys-typescale-body-medium-font, var(--md-ref-typeface-plain, Roboto)));font-size:var(--md-list-item-supporting-text-size, var(--md-sys-typescale-body-medium-size, 0.875rem));line-height:var(--md-list-item-supporting-text-line-height, var(--md-sys-typescale-body-medium-line-height, 1.25rem));font-weight:var(--md-list-item-supporting-text-weight, var(--md-sys-typescale-body-medium-weight, var(--md-ref-typeface-weight-regular, 400)))}[slot=trailing-supporting-text]{color:var(--md-list-item-trailing-supporting-text-color, var(--md-sys-color-on-surface-variant, #49454f));font-family:var(--md-list-item-trailing-supporting-text-font, var(--md-sys-typescale-label-small-font, var(--md-ref-typeface-plain, Roboto)));font-size:var(--md-list-item-trailing-supporting-text-size, var(--md-sys-typescale-label-small-size, 0.6875rem));line-height:var(--md-list-item-trailing-supporting-text-line-height, var(--md-sys-typescale-label-small-line-height, 1rem));font-weight:var(--md-list-item-trailing-supporting-text-weight, var(--md-sys-typescale-label-small-weight, var(--md-ref-typeface-weight-medium, 500)))}:is([slot=start],[slot=end])::slotted(*){fill:currentColor}[slot=start]{color:var(--md-list-item-leading-icon-color, var(--md-sys-color-on-surface-variant, #49454f))}[slot=end]{color:var(--md-list-item-trailing-icon-color, var(--md-sys-color-on-surface-variant, #49454f))}@media(forced-colors: active){.disabled slot{color:GrayText}.list-item.disabled{color:GrayText;opacity:1}}
`
/**
 * @license
 * Copyright 2022 Google LLC
 * SPDX-License-Identifier: Apache-2.0
 */;let Si=class extends ki{};Si.styles=[Ii],Si=t([e("md-list-item")],Si),console.log("Material Web Components 已加载");
