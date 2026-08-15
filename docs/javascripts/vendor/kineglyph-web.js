//#region ../../node_modules/animejs/dist/modules/core/consts.js
var e = typeof window < "u", t = e ? window : null, n = e ? document : null, r = {
	OBJECT: 0,
	ATTRIBUTE: 1,
	CSS: 2,
	TRANSFORM: 3,
	CSS_VAR: 4
}, i = {
	NUMBER: 0,
	UNIT: 1,
	COLOR: 2,
	COMPLEX: 3
}, a = {
	NONE: 0,
	AUTO: 1,
	FORCE: 2
}, o = {
	replace: 0,
	none: 1,
	blend: 2
}, s = Symbol(), c = 1e-11, l = 0xe8d4a51000, u = 1e3, d = [
	"perspective",
	"translateX",
	"translateY",
	"translateZ",
	"rotate",
	"rotateX",
	"rotateY",
	"rotateZ",
	"scale",
	"scaleX",
	"scaleY",
	"scaleZ",
	"skew",
	"skewX",
	"skewY"
], f = /*#__PURE__*/ d.reduce((e, t) => ({
	...e,
	[t]: t + "("
}), {}), p = () => {}, m = {
	id: null,
	keyframes: null,
	playbackEase: null,
	playbackRate: 1,
	frameRate: 240,
	loop: 0,
	reversed: !1,
	alternate: !1,
	autoplay: !0,
	persist: !1,
	duration: u,
	delay: 0,
	loopDelay: 0,
	ease: "out(2)",
	composition: o.replace,
	modifier: (e) => e,
	onBegin: p,
	onBeforeUpdate: p,
	onUpdate: p,
	onLoop: p,
	onPause: p,
	onComplete: p,
	onRender: p
}, h = {
	current: null,
	root: n
}, g = {
	defaults: m,
	precision: 4,
	timeScale: 1,
	tickThreshold: 200,
	editor: null
}, _ = {
	version: "4.5.0",
	engine: null
};
e && (t.AnimeJS ||= [], t.AnimeJS.push(_));
//#endregion
//#region ../../node_modules/animejs/dist/modules/core/helpers.js
var v = Date.now, y = Array.isArray, b = (e) => typeof e == "string", x = (e) => typeof e == "function", S = (e) => e === void 0, C = (e) => S(e) || e === null, w = Math.floor, T = Math.round, E = (e, t, n) => e < t ? t : e > n ? n : e, D = (e, t) => {
	if (t < 0) return e;
	if (!t) return T(e);
	let n = 10 ** t;
	return T(e * n) / n;
}, O = (e, t, n) => n === 1 ? t : n === 0 ? e : e + (t - e) * n, k = (e) => e === Infinity ? l : e === -Infinity ? -l : e, A = (e) => e <= 1e-11 ? c : k(D(e, 11)), j = (e) => y(e) ? [...e] : e, M = (e, t) => {
	let n = { ...e };
	for (let r in t) {
		let i = e[r];
		n[r] = S(i) ? t[r] : i;
	}
	return n;
}, N = (e, t, n, r = "_prev", i = "_next") => {
	let a = e._head, o = i;
	for (n && (a = e._tail, o = r); a;) {
		let e = a[o];
		t(a), a = e;
	}
}, P = (e, t, n = "_prev", r = "_next") => {
	let i = t[n], a = t[r];
	i ? i[r] = a : e._head = a, a ? a[n] = i : e._tail = i, t[n] = null, t[r] = null;
}, ee = (e, t, n, r = "_prev", i = "_next") => {
	let a = e._tail;
	for (; a && n && n(a, t);) a = a[r];
	let o = a ? a[i] : e._head;
	a ? a[i] = t : e._head = t, o ? o[r] = t : e._tail = t, t[r] = a, t[i] = o;
}, te = (e) => {
	let t = "";
	for (let n = 0, r = d.length; n < r; n++) {
		let r = d[n], i = e[r];
		if (i !== void 0) {
			if (r === "translateX") {
				let r = e.translateY;
				if (r !== void 0) {
					let a = e.translateZ;
					a === void 0 ? (t += `translate(${i},${r}) `, n += 1) : (t += `translate3d(${i},${r},${a}) `, n += 2);
					continue;
				}
			}
			if (r === "scaleX" && e.scale === void 0) {
				let r = e.scaleY;
				if (r !== void 0) {
					let a = e.scaleZ;
					a === void 0 ? (t += `scale(${i},${r}) `, n += 1) : (t += `scale3d(${i},${r},${a}) `, n += 2);
					continue;
				}
			}
			t += `${f[r]}${i}) `;
		}
		r === "rotateZ" && e.rotate3d !== void 0 && (t += `rotate3d(${e.rotate3d}) `);
	}
	return e.matrix !== void 0 && (t += `matrix(${e.matrix}) `), e.matrix3d !== void 0 && (t += `matrix3d(${e.matrix3d}) `), t;
}, F = (e, t) => S(e) ? t : e;
i.NUMBER;
var ne = (e, t, n) => {
	let r = e._modifier, i = e._fromNumbers, a = e._toNumbers, o = e._strings, s = o[0];
	for (let c = 0, l = a.length; c < l; c++) {
		let l = r(D(O(i[c], a[c], t), n)), u = o[c + 1];
		s += `${u ? l + u : l}`, e._numbers[c] = l;
	}
	return s;
}, re = (e, t, n, l, u) => {
	let d = e.parent, f = e.duration, p = e.completed, m = e.iterationDuration, h = e.iterationCount, _ = e._currentIteration, v = e._loopDelay, y = e._reversed, b = e._alternate, x = e._hasChildren, S = e._delay, C = e._currentTime, w = S + m, T = t - S, k = E(C, -S, f), A = E(T, -S, f), j = T - C, M = A > 0, N = A >= f, P = f <= c, ee = u === a.FORCE, F = 0, re = T, ie = 0;
	if (h > 1) {
		let t = m + (N ? 0 : v), n = ~~(A / t);
		e._currentIteration = E(n, 0, h), N && e._currentIteration--, F = e._currentIteration % 2, re = A - n * t || 0;
	}
	let ae = y ^ (b && F), oe = e._ease, se = N ? ae ? 0 : f : ae ? m - re : re;
	oe && (se = m * oe(se / m) || 0);
	let ce = (d ? d.backwards : T < C) ? !ae : !!ae;
	if (e._currentTime = T, e._iterationTime = se, e.backwards = ce, M && !e.began ? (e.began = !0, !n && !(d && (ce || !d.began)) && e.onBegin(e)) : T <= 0 && (e.began = !1), !n && !x && M && e._currentIteration !== _ && e.onLoop(e), ee || u === a.AUTO && (t >= (d && S > 0 ? 0 : S) && t <= w || t <= S && k > S || t >= w && k !== f) || se >= w && k !== f || se <= S && k > 0 && !N || t <= k && k === f && p || N && !p && P) {
		if (M && (e.computeDeltaTime(k), n || e.onBeforeUpdate(e)), !x) {
			let t = ee || (ce ? j * -1 : j) >= g.tickThreshold, a = D(e._offset + (d ? d._offset : 0) + S + se, 12), c = e._head, u, f, p, m, h = 0;
			for (; c;) {
				let e = c._composition, n = c._currentTime, d = c._changeDuration, _ = c._absoluteStartTime + c._changeDuration, v = c._nextRep, y = c._prevRep, b = e !== o.none, x = y ? y._absoluteStartTime + y._changeDuration : 0, S = y && y.parent !== c.parent, C = !v || v._isOverridden ? _ : v.parent === c.parent ? _ + v._delay : v._absoluteStartTime < v._absoluteUpdateStartTime ? v._absoluteStartTime : v._absoluteUpdateStartTime;
				if ((t || (n !== d || a <= C || y && !S && (!v || v.parent !== c.parent)) && (n !== 0 || a >= c._absoluteStartTime || S && !c._hasFromValue && !y._isOverridden && a >= x || v && !v._isOverridden && v.parent === c.parent && v._currentTime !== 0 && se < v._startTime)) && (!y || S || se >= c._startTime) && (!b || !c._isOverridden && (!c._isOverlapped || a <= _) && (!v || v._isOverridden || a <= C) && (!y || y._isOverridden || (S ? a >= c._absoluteStartTime || !c._hasFromValue && a >= x : a >= x + c._delay)))) {
					let t = c._currentTime = E(se - c._startTime, 0, d), n = c._ease(t / c._updateDuration), a = c._modifier, _ = c._valueType, v = c._tweenType, y = v === r.OBJECT, x = _ === i.NUMBER, S = x && y || n === 0 || n === 1 ? -1 : g.precision, C, w;
					if (x) C = w = a(D(O(c._fromNumber, c._toNumber, n), S));
					else if (_ === i.UNIT) w = a(D(O(c._fromNumber, c._toNumber, n), S)), C = `${w}${c._unit}`;
					else if (_ === i.COLOR) {
						let e = c._numbers, t = c._fromNumbers, r = c._toNumbers, i = 1 - n, o = t[0], s = t[1], u = t[2], d = r[0], f = r[1], p = r[2];
						e[0] = a(Math.sqrt(o * o * i + d * d * n)), e[1] = a(Math.sqrt(s * s * i + f * f * n)), e[2] = a(Math.sqrt(u * u * i + p * p * n)), e[3] = a(O(t[3], r[3], n)), (!c._setter || l) && (C = `rgba(${D(e[0], 0)},${D(e[1], 0)},${D(e[2], 0)},${e[3]})`);
					} else _ === i.COMPLEX && (C = ne(c, n, S));
					if (b && (c._number = w), !l && e !== o.blend) {
						let e = c.property;
						u = c.target, c._setter ? c._setter(u, w, c) : y ? u[e] = C : v === r.ATTRIBUTE ? u.setAttribute(e, C) : (f = u.style, v === r.TRANSFORM ? (u !== p && (p = u, m = u[s]), m[e] = C, h = 1) : v === r.CSS ? f[e] = C : v === r.CSS_VAR && f.setProperty(e, C)), M && (ie = 1);
					} else c._value = C;
				} else n && y && !S && se < c._startTime && (c._currentTime = 0);
				h && c._renderTransforms && (f.transform = te(m), h = 0), c = c._next;
			}
			!n && ie && e.onRender(e);
		}
		!n && M && e.onUpdate(e);
	}
	return d && P ? !n && (d.began && !ce && T > 0 && !p || ce && T <= 1e-11 && p) && (e.onComplete(e), e.completed = !ce) : M && N ? h === Infinity ? e._startTime += e.duration : e._currentIteration >= h - 1 && (e.paused = !0, !p && !x && (e.completed = !0, !n && !(d && (ce || !d.began)) && (e.onComplete(e), e._resolve(e)))) : e.completed = !1, ie;
}, ie = (e, t, n, r, i) => {
	let o = e._currentIteration;
	if (re(e, t, n, r, i), e._hasChildren) {
		let s = e, c = s.backwards, l = r ? t : s._iterationTime, u = v(), d = 0, f = !0;
		if (!r && s._currentIteration !== o) {
			let e = s.iterationDuration;
			N(s, (t) => {
				if (!c) !t.completed && !t.backwards && t._currentTime < t.iterationDuration && re(t, e, n, 1, a.FORCE), t.began = !1, t.completed = !1;
				else {
					let r = t.duration, i = t._offset + t._delay, a = i + r;
					!n && r <= 1e-11 && (!i || a === e) && t.onComplete(t);
				}
			}), n || s.onLoop(s);
		}
		N(s, (e) => {
			let t = D((l - e._offset) * e._speed, 12);
			if (c && t > e._delay + e.duration) return;
			let a = e._fps < s._fps ? e.requestTick(u) : i;
			d += re(e, t, n, r, a), !e.completed && f && (f = !1);
		}, c), !n && d && s.onRender(s), (f || c) && s._currentTime >= s.duration && (s.paused = !0, s.completed || (s.completed = !0, n || (s.onComplete(s), s._resolve(s))));
	}
}, ae = class {
	constructor(e = 0) {
		this.deltaTime = 0, this._currentTime = e, this._lastTickTime = e, this._startTime = e, this._lastTime = e, this._frameDuration = u / 240, this._fps = 240, this._speed = 1, this._hasChildren = !1, this._head = null, this._tail = null;
	}
	get fps() {
		return this._fps;
	}
	set fps(e) {
		let t = +e, n = t < 1e-11 ? c : t, r = u / n;
		n > m.frameRate && (m.frameRate = n), this._fps = n, this._frameDuration = r;
	}
	get speed() {
		return this._speed;
	}
	set speed(e) {
		let t = +e;
		this._speed = t < 1e-11 ? c : t;
	}
	requestTick(e) {
		let t = this._frameDuration, n = e - this._lastTickTime, r = t * .25;
		return n + (r < 4 ? r : 4) < t ? a.NONE : (this._lastTickTime = n >= t ? e - n % t : e, a.AUTO);
	}
	computeDeltaTime(e) {
		let t = e - this._lastTime;
		return this.deltaTime = t, this._lastTime = e, t;
	}
}, oe = {
	animation: null,
	update: p
}, se = (e) => {
	let t = oe.animation;
	return t || (t = {
		duration: c,
		computeDeltaTime: p,
		_offset: 0,
		_delay: 0,
		_head: null,
		_tail: null
	}, oe.animation = t, oe.update = () => {
		e.forEach((e) => {
			for (let t in e) {
				let n = e[t], r = n._head;
				if (r) {
					let e = r._valueType, t = e === i.COMPLEX || e === i.COLOR ? j(r._fromNumbers) : null, a = r._fromNumber, o = n._tail;
					for (; o && o !== r;) {
						if (t) for (let e = 0, n = o._numbers.length; e < n; e++) t[e] += o._numbers[e];
						else a += o._number;
						o = o._prevAdd;
					}
					r._toNumber = a, r._toNumbers = t;
				}
			}
		}), re(t, 1, 1, 0, a.FORCE);
	}), t;
}, ce = e ? requestAnimationFrame : setImmediate, le = e ? cancelAnimationFrame : clearImmediate, ue = class extends ae {
	constructor(e) {
		super(e), this.useDefaultMainLoop = !0, this.pauseOnDocumentHidden = !0, this.defaults = m, this.paused = !0, this.reqId = 0;
	}
	update() {
		let e = this._currentTime = v();
		if (this.requestTick(e)) {
			this.computeDeltaTime(e);
			let t = this._speed, n = this._fps, r = this._head;
			for (; r;) {
				let i = r._next;
				r.paused ? (P(this, r), this._hasChildren = !!this._tail, r._running = !1, r.completed && !r._cancelled && r.cancel()) : ie(r, (e - r._startTime) * r._speed * t, 0, 0, r._fps < n ? r.requestTick(e) : a.AUTO), r = i;
			}
			oe.update();
		}
	}
	wake() {
		return this.useDefaultMainLoop && !this.reqId && (this.requestTick(v()), this.reqId = ce(fe)), this;
	}
	pause() {
		if (this.reqId) return this.paused = !0, pe();
	}
	resume() {
		if (this.paused) return this.paused = !1, N(this, (e) => e.resetTime()), this.wake();
	}
	get speed() {
		return this._speed * (g.timeScale === 1 ? 1 : u);
	}
	set speed(e) {
		let t = e * g.timeScale;
		this._speed !== t && (this._speed = t, N(this, (e) => e.speed = e._speed));
	}
	get timeUnit() {
		return g.timeScale === 1 ? "ms" : "s";
	}
	set timeUnit(e) {
		let t = .001, n = e === "s", r = n ? t : 1;
		if (g.timeScale !== r) {
			g.timeScale = r, g.tickThreshold = 200 * r;
			let e = n ? t : u;
			this.defaults.duration *= e, this._speed *= e;
		}
	}
	get precision() {
		return g.precision;
	}
	set precision(e) {
		g.precision = e;
	}
}, de = /*#__PURE__*/ (() => {
	let t = new ue(v());
	return e && (_.engine = t, n.addEventListener("visibilitychange", () => {
		t.pauseOnDocumentHidden && (n.hidden ? t.pause() : t.resume());
	})), t;
})(), fe = () => {
	de._head ? (de.reqId = ce(fe), de.update()) : de.reqId = 0;
}, pe = () => (le(de.reqId), de.reqId = 0, de), me = {
	_rep: /* @__PURE__ */ new WeakMap(),
	_add: /* @__PURE__ */ new Map()
}, I = (e, t, n = "_rep") => {
	let r = me[n], i = r.get(e);
	return i || (i = {}, r.set(e, i)), i[t] ? i[t] : i[t] = {
		_head: null,
		_tail: null
	};
}, L = (e, t) => e._isOverridden || e._absoluteStartTime > t._absoluteStartTime, he = (e) => {
	e._isOverlapped = 1, e._isOverridden = 1, e._changeDuration = c, e._currentTime = c;
}, ge = (e, t) => {
	let n = e._composition;
	if (n === o.replace) {
		let n = e._absoluteStartTime;
		ee(t, e, L, "_prevRep", "_nextRep");
		let r = e._prevRep;
		if (r) {
			let t = r.parent, i = r._absoluteEndTime;
			if (e.parent.id !== t.id && t.iterationCount > 1 && i + (t.duration - t.iterationDuration) > n) {
				he(r);
				let e = r._prevRep;
				for (; e && e.parent.id === t.id;) he(e), e = e._prevRep;
			}
			let a = e._absoluteUpdateStartTime;
			if (i > a) {
				let e = r._startTime, t = D(a - (i - (e + r._updateDuration)) - e, 12);
				r._changeDuration = t, r._currentTime = t, r._isOverlapped = 1, t < 1e-11 && he(r);
			}
			let o = e.parent.parent;
			if (!o || o !== t.parent) {
				let e = !0;
				if (N(t, (t) => {
					t._isOverlapped || (e = !1);
				}), e) {
					let e = t.parent;
					if (e) {
						let n = !0;
						N(e, (e) => {
							e !== t && N(e, (e) => {
								e._isOverlapped || (n = !1);
							});
						}), n && e.cancel();
					} else t.cancel();
				}
			}
		}
	} else if (n === o.blend) {
		let t = I(e.target, e.property, "_add"), n = se(me._add), r = t._head;
		r || (r = { ...e }, r._composition = o.replace, r._updateDuration = c, r._startTime = 0, r._numbers = j(e._fromNumbers), r._number = 0, r._next = null, r._prev = null, ee(t, r), ee(n, r));
		let i = e._toNumber;
		if (e._fromNumber = r._fromNumber - i, e._toNumber = 0, e._numbers = j(e._fromNumbers), e._number = 0, r._fromNumber = i, e._toNumbers.length) {
			let t = j(e._toNumbers);
			t.forEach((t, n) => {
				e._fromNumbers[n] = r._fromNumbers[n] - t, e._toNumbers[n] = 0;
			}), r._fromNumbers = t;
		}
		ee(t, e, null, "_prevAdd", "_nextAdd");
	}
	return e;
}, _e = (e) => {
	let t = e._composition;
	if (t !== o.none) {
		let n = e.target, r = e.property, i = me._rep.get(n)[r];
		if (P(i, e, "_prevRep", "_nextRep"), t === o.blend) {
			let t = me._add, i = t.get(n);
			if (!i) return;
			let a = i[r], o = oe.animation;
			P(a, e, "_prevAdd", "_nextAdd");
			let s = a._head;
			if (s && s === a._tail) {
				P(a, s, "_prevAdd", "_nextAdd"), P(o, s);
				let e = !0;
				for (let t in i) if (i[t]._head) {
					e = !1;
					break;
				}
				e && t.delete(n);
			}
		}
	}
	return e;
}, ve = (e) => (e.paused = !0, e.began = !1, e.completed = !1, e), ye = (e) => e._cancelled ? (e._hasChildren ? N(e, ye) : N(e, (e) => {
	e._composition !== o.none && ge(e, I(e.target, e.property));
}), e._cancelled = 0, e) : e, be = 0, xe = (e, t) => e._priority > t._priority, Se = class extends ae {
	constructor(e = {}, t = null, n = 0) {
		super(0), ++be;
		let { id: r, delay: i, duration: a, reversed: o, alternate: s, loop: c, loopDelay: l, autoplay: u, frameRate: d, playbackRate: f, priority: m, onComplete: _, onLoop: y, onPause: b, onBegin: C, onBeforeUpdate: w, onUpdate: T } = e;
		h.current && h.current.register(this);
		let E = t ? 0 : de._lastTickTime, D = t ? t.defaults : g.defaults, O = x(i) || S(i) ? D.delay : +i, A = x(a) || S(a) ? Infinity : +a, j = F(c, D.loop), M = F(l, D.loopDelay), N = j === !0 || j === Infinity || j < 0 ? Infinity : j + 1, P = 0;
		t ? P = n : (de.reqId || de.requestTick(v()), P = (de._lastTickTime - de._startTime) * g.timeScale), this.id = S(r) ? be : r, this.parent = t, this.duration = k((A + M) * N - M) || 1e-11, this.backwards = !1, this.paused = !0, this.began = !1, this.completed = !1, this.onBegin = C || D.onBegin, this.onBeforeUpdate = w || D.onBeforeUpdate, this.onUpdate = T || D.onUpdate, this.onLoop = y || D.onLoop, this.onPause = b || D.onPause, this.onComplete = _ || D.onComplete, this.iterationDuration = A, this.iterationCount = N, this._autoplay = !t && F(u, D.autoplay), this._offset = P, this._delay = O, this._loopDelay = M, this._iterationTime = 0, this._currentIteration = 0, this._resolve = p, this._running = !1, this._reversed = +F(o, D.reversed), this._reverse = this._reversed, this._cancelled = 0, this._alternate = F(s, D.alternate), this._prev = null, this._next = null, this._lastTickTime = E, this._startTime = E, this._lastTime = E, this._fps = F(d, D.frameRate), this._speed = F(f, D.playbackRate), this._priority = +F(m, 1);
	}
	get cancelled() {
		return !!this._cancelled;
	}
	set cancelled(e) {
		e ? this.cancel() : this.reset(!0).play();
	}
	get currentTime() {
		return E(D(this._currentTime, g.precision), -this._delay, this.duration);
	}
	set currentTime(e) {
		let t = this.paused;
		this.pause().seek(+e), t || this.resume();
	}
	get iterationCurrentTime() {
		return E(D(this._iterationTime, g.precision), 0, this.iterationDuration);
	}
	set iterationCurrentTime(e) {
		this.currentTime = this.iterationDuration * this._currentIteration + e;
	}
	get progress() {
		return E(D(this._currentTime / this.duration, 10), 0, 1);
	}
	set progress(e) {
		this.currentTime = this.duration * e;
	}
	get iterationProgress() {
		return E(D(this._iterationTime / this.iterationDuration, 10), 0, 1);
	}
	set iterationProgress(e) {
		let t = this.iterationDuration;
		this.currentTime = t * this._currentIteration + t * e;
	}
	get currentIteration() {
		return this._currentIteration;
	}
	set currentIteration(e) {
		this.currentTime = this.iterationDuration * E(+e, 0, this.iterationCount - 1);
	}
	get reversed() {
		return !!this._reversed;
	}
	set reversed(e) {
		e ? this.reverse() : this.play();
	}
	get speed() {
		return super.speed;
	}
	set speed(e) {
		super.speed = e, this.resetTime();
	}
	reset(e = !1) {
		return ye(this), this._reversed && !this._reverse && (this.reversed = !1), this._iterationTime = this.iterationDuration, ie(this, 0, 1, ~~e, a.FORCE), ve(this), this._hasChildren && N(this, ve), this;
	}
	init(e = !1) {
		this.fps = this._fps, this.speed = this._speed, !e && this._hasChildren && ie(this, this.duration, 1, ~~e, a.FORCE), this.reset(e);
		let t = this._autoplay;
		return t === !0 ? this.resume() : t && !S(t.linked) && t.link(this), this;
	}
	resetTime() {
		let e = 1 / (this._speed * de._speed);
		return this._startTime = v() - (this._currentTime + this._delay) * e, this;
	}
	pause() {
		return this.paused ? this : (this.paused = !0, this.onPause(this), this);
	}
	resume() {
		return this.paused ? (this.paused = !1, this.duration <= 1e-11 && !this._hasChildren ? ie(this, c, 0, 0, a.FORCE) : (this._running ||= (ee(de, this, xe), de._hasChildren = !0, !0), this.resetTime(), this._startTime -= 12, de.wake()), this) : this;
	}
	restart() {
		return this.reset().resume();
	}
	seek(e, t = 0, n = 0) {
		ye(this), this.completed = !1;
		let r = this.paused;
		return this.paused = !0, ie(this, e + this._delay, ~~t, ~~n, a.AUTO), r ? this : this.resume();
	}
	alternate() {
		let e = this._reversed, t = this.iterationCount, n = this.iterationDuration, r = t === Infinity ? w(l / n) : t;
		return this._reversed = +(this._alternate && !(r % 2) ? e : !e), t === Infinity ? this.iterationProgress = this._reversed ? 1 - this.iterationProgress : this.iterationProgress : this.seek(n * r - this._currentTime), this.resetTime(), this;
	}
	play() {
		return this._reversed && this.alternate(), this.resume();
	}
	reverse() {
		return this._reversed || this.alternate(), this.resume();
	}
	cancel() {
		return this._hasChildren ? N(this, (e) => e.cancel(), !0) : N(this, _e), this._cancelled = 1, this.pause();
	}
	stretch(e) {
		let t = this.duration, n = A(e);
		if (t === n) return this;
		let r = e / t, i = e <= c;
		return this.duration = i ? c : n, this.iterationDuration = i ? c : A(this.iterationDuration * r), this._offset *= r, this._delay *= r, this._loopDelay *= r, this;
	}
	revert() {
		ie(this, 0, 1, 0, a.AUTO);
		let e = this._autoplay;
		return e && e.linked && e.linked === this && e.revert(), this.cancel();
	}
	complete(e = 0) {
		return this.seek(this.duration, e).cancel();
	}
	then(e = p) {
		let t = this.then, n = () => {
			this.then = null, e(this), this.then = t, this._resolve = p;
		};
		return new Promise((e) => (this._resolve = () => e(n()), this.completed && this._resolve(), this));
	}
}, Ce = (e) => new Se(e, null, 0).init();
//#endregion
//#region ../../node_modules/animejs/dist/modules/core/targets.js
function we(e) {
	let t = b(e) ? h.root.querySelectorAll(e) : e;
	if (t instanceof NodeList || t instanceof HTMLCollection) return t;
}
function Te(t) {
	if (C(t)) return [];
	if (!e) return y(t) && t.flat(Infinity) || [t];
	if (y(t)) {
		let e = t.flat(Infinity), n = [];
		for (let t = 0, r = e.length; t < r; t++) {
			let r = e[t];
			if (!C(r)) {
				let e = we(r);
				if (e) for (let t = 0, r = e.length; t < r; t++) {
					let r = e[t];
					if (!C(r)) {
						let e = !1;
						for (let t = 0, i = n.length; t < i; t++) if (n[t] === r) {
							e = !0;
							break;
						}
						e || n.push(r);
					}
				}
				else {
					let e = !1;
					for (let t = 0, i = n.length; t < i; t++) if (n[t] === r) {
						e = !0;
						break;
					}
					e || n.push(r);
				}
			}
		}
		return n;
	}
	let n = we(t);
	return n ? Array.from(n) : [t];
}
//#endregion
//#region ../../node_modules/animejs/dist/modules/utils/time.js
var Ee = (e) => {
	let t;
	return ((...n) => {
		let r, i, a, o, s;
		t && (r = t.currentIteration, i = t.iterationProgress, a = t.reversed, o = t._alternate, s = t._startTime, t.revert());
		let c = e(...n);
		return c && !x(c) && c.revert && (t = c), S(i) || (t.currentIteration = r, t.iterationProgress = (o && r % 2 ? !a : a) ? 1 - i : i, t._startTime = s), c || p;
	});
}, De = class {
	constructor(e = {}) {
		h.current && h.current.register(this);
		let r = e.root, i = n;
		r && (i = r.current || r.nativeElement || Te(r)[0] || n);
		let a = e.defaults, o = g.defaults, s = e.mediaQueries;
		if (this.defaults = a ? M(a, o) : o, this.root = i, this.constructors = [], this.revertConstructors = [], this.revertibles = [], this.constructorsOnce = [], this.revertConstructorsOnce = [], this.revertiblesOnce = [], this.once = !1, this.onceIndex = 0, this.methods = {}, this.matches = {}, this.mediaQueryLists = {}, this.data = {}, s) for (let e in s) {
			let n = t.matchMedia(s[e]);
			this.mediaQueryLists[e] = n, n.addEventListener("change", this);
		}
	}
	register(e) {
		(this.once ? this.revertiblesOnce : this.revertibles).push(e);
	}
	execute(e) {
		let t = h.current, n = h.root, r = g.defaults;
		h.current = this, h.root = this.root, g.defaults = this.defaults;
		let i = this.mediaQueryLists;
		for (let e in i) this.matches[e] = i[e].matches;
		let a = e(this);
		return h.current = t, h.root = n, g.defaults = r, a;
	}
	refresh() {
		return this.onceIndex = 0, this.execute(() => {
			let e = this.revertibles.length, t = this.revertConstructors.length;
			for (; e--;) this.revertibles[e].revert();
			for (; t--;) this.revertConstructors[t](this);
			this.revertibles.length = 0, this.revertConstructors.length = 0, this.constructors.forEach((e) => {
				let t = e(this);
				x(t) && this.revertConstructors.push(t);
			});
		}), this;
	}
	add(e, t) {
		if (this.once = !1, x(e)) {
			let t = e;
			this.constructors.push(t), this.execute(() => {
				let e = t(this);
				x(e) && this.revertConstructors.push(e);
			});
		} else this.methods[e] = (...e) => this.execute(() => t(...e));
		return this;
	}
	addOnce(e) {
		if (this.once = !0, x(e)) {
			let t = this.onceIndex++;
			if (this.constructorsOnce[t]) return this;
			let n = e;
			this.constructorsOnce[t] = n, this.execute(() => {
				let e = n(this);
				x(e) && this.revertConstructorsOnce.push(e);
			});
		}
		return this;
	}
	keepTime(e) {
		this.once = !0;
		let t = this.onceIndex++, n = this.constructorsOnce[t];
		if (x(n)) return n(this);
		let r = Ee(e);
		this.constructorsOnce[t] = r;
		let i;
		return this.execute(() => {
			i = r(this);
		}), i;
	}
	handleEvent(e) {
		e.type === "change" && this.refresh();
	}
	revert() {
		let e = this.revertibles, t = this.revertConstructors, n = this.revertiblesOnce, r = this.revertConstructorsOnce, i = this.mediaQueryLists, a = e.length, o = t.length, s = n.length, c = r.length;
		for (; a--;) e[a].revert();
		for (; o--;) t[o](this);
		for (; s--;) n[s].revert();
		for (; c--;) r[c](this);
		for (let e in i) i[e].removeEventListener("change", this);
		e.length = 0, t.length = 0, this.constructors.length = 0, n.length = 0, r.length = 0, this.constructorsOnce.length = 0, this.onceIndex = 0, this.matches = {}, this.methods = {}, this.mediaQueryLists = {}, this.data = {};
	}
}, Oe = (e) => new De(e);
//#endregion
//#region ../core/dist/authoring.js
function ke(e, t, n, r, i, a = n) {
	let o = [];
	return e > 0 && o.push({
		time: 0,
		value: a
	}), o.push({
		time: e,
		value: n
	}), o.push({
		time: t,
		value: r,
		easing: i
	}), o;
}
function Ae(e, t, n, r = `${e}:${t}`) {
	return {
		id: r,
		target: e,
		property: t,
		keyframes: n
	};
}
function je(e, t, n, r = 0) {
	return Ae(e, "opacity", ke(t, n, r, 1, "easeOut"));
}
function Me(e, t, n, r = .94) {
	return Ae(e, "scale", ke(t, n, r, 1, "easeOut"));
}
function Ne(e, t, n, r, i = "y") {
	return Ae(e, i === "x" ? "translateX" : "translateY", ke(t, n, r, 0, "easeOut"));
}
function Pe(e, t, n, r = {}) {
	let i = [je(e, t, n)];
	return r.scale !== void 0 && i.push(Me(e, t, n, r.scale)), r.offset !== void 0 && i.push(Ne(e, t, n, r.offset)), i;
}
function Fe(e, t, n) {
	return [Ae(e, "opacity", [
		{
			time: 0,
			value: 0
		},
		{
			time: Math.max(0, t - 1),
			value: 0
		},
		{
			time: t,
			value: 1
		}
	]), Ae(e, "edgeReveal", ke(t, n, 0, 1, "easeInOut"))];
}
function Ie(e, t, n) {
	let r = [
		{
			time: 0,
			value: 0
		},
		{
			time: t,
			value: 0
		},
		{
			time: t + 1,
			value: 1
		}
	];
	return n !== void 0 && r.push({
		time: n,
		value: 1
	}, {
		time: n + 200,
		value: 0,
		easing: "easeOut"
	}), Ae(e, "flow", r);
}
function Le(e, t, n, r = 1, i = r) {
	return Ae(e, "highlight", Be([
		{
			time: 0,
			value: 0
		},
		{
			time: t,
			value: 0
		},
		{
			time: (t + n) / 2,
			value: r,
			easing: "easeOut"
		},
		{
			time: n,
			value: i,
			easing: "easeInOut"
		}
	]));
}
function Re(e, t, n = 500) {
	return Ae(e, "highlight", Be([
		{
			time: 0,
			value: 0
		},
		{
			time: t,
			value: 0
		},
		{
			time: t + n / 2,
			value: 1,
			easing: "easeOut"
		},
		{
			time: t + n,
			value: 0,
			easing: "easeIn"
		}
	]), `${e}:highlight:${t}`);
}
function ze(e, t, n, r = 0, i = 1) {
	return Ae(e, "progress", ke(t, n, r, i, "easeInOut"));
}
function Be(e) {
	let t = [];
	for (let n of e) {
		let e = t[t.length - 1];
		if (e !== void 0 && n.time <= e.time) {
			t[t.length - 1] = {
				...n,
				time: e.time
			};
			continue;
		}
		t.push(n);
	}
	return t;
}
function Ve(e, t) {
	let n = [];
	for (let t of e) Array.isArray(t) ? n.push(...t) : n.push(t);
	let r = /* @__PURE__ */ new Set(), i = 0;
	for (let e of n) {
		if (r.has(e.id)) throw Error(`duplicate timeline track id: ${e.id}`);
		r.add(e.id);
		for (let t of e.keyframes) i = Math.max(i, t.time);
	}
	let a = t ?? i;
	if (a < i) throw RangeError(`timeline duration ${a} is shorter than its last keyframe ${i}`);
	return {
		duration: a,
		tracks: n
	};
}
//#endregion
//#region ../core/dist/easing.js
function He(e, t) {
	if (!Number.isFinite(e)) throw RangeError(`${t} must be finite`);
	return e;
}
function Ue(e, t, n, r) {
	if (He(e, "x1"), He(t, "y1"), He(n, "x2"), He(r, "y2"), e < 0 || e > 1 || n < 0 || n > 1) throw RangeError("cubic Bézier x control points must be between 0 and 1");
	return {
		type: "cubic-bezier",
		x1: e,
		y1: t,
		x2: n,
		y2: r
	};
}
function We(e = {}) {
	let t = He(e.frequency ?? 10.5, "spring frequency"), n = He(e.damping ?? 7, "spring damping");
	if (t <= 0) throw RangeError("spring frequency must be greater than zero");
	if (n < 0) throw RangeError("spring damping must be non-negative");
	return {
		type: "spring",
		frequency: t,
		damping: n
	};
}
function Ge(e) {
	return Math.min(1, Math.max(0, e));
}
function Ke(e, t, n) {
	let r = 1 - n;
	return 3 * r * r * n * e + 3 * r * n * n * t + n * n * n;
}
function qe(e, t, n) {
	let r = 1 - n;
	return 3 * r * r * e + 6 * r * n * (t - e) + 3 * n * n * (1 - t);
}
function Je(e, t) {
	let n = t;
	for (let r = 0; r < 8; r += 1) {
		let r = Ke(e.x1, e.x2, n) - t;
		if (Math.abs(r) < 1e-7) break;
		let i = qe(e.x1, e.x2, n);
		if (Math.abs(i) < 1e-7) break;
		n = Ge(n - r / i);
	}
	let r = 0, i = 1;
	for (let a = 0; a < 12; a += 1) {
		let a = Ke(e.x1, e.x2, n);
		if (Math.abs(a - t) < 1e-7) break;
		a < t ? r = n : i = n, n = (r + i) / 2;
	}
	return Ke(e.y1, e.y2, n);
}
function Ye(e, t) {
	let n = 1 - Math.exp(-e.damping * t) * Math.cos(e.frequency * t), r = 1 - Math.exp(-e.damping) * Math.cos(e.frequency);
	return Math.abs(r) < 1e-9 ? n : n / r;
}
function Xe(e, t) {
	let n = Ge(t);
	if (n === 0 || n === 1) return n;
	if (typeof e == "object") return e.type === "cubic-bezier" ? Je(e, n) : Ye(e, n);
	switch (e ?? "linear") {
		case "easeIn": return n * n;
		case "easeOut": return 1 - (1 - n) ** 2;
		case "easeInOut": return n < .5 ? 2 * n * n : 1 - (-2 * n + 2) ** 2 / 2;
		case "easeInCubic": return n ** 3;
		case "easeOutCubic": return 1 - (1 - n) ** 3;
		case "easeInOutCubic": return n < .5 ? 4 * n ** 3 : 1 - (-2 * n + 2) ** 3 / 2;
		case "easeOutBack": return 1 + 2.70158 * (n - 1) ** 3 + 1.70158 * (n - 1) ** 2;
		case "easeOutExpo": return 1 - 2 ** (-10 * n);
		case "linear": return n;
	}
}
//#endregion
//#region ../core/dist/geometry.js
var Ze = 24;
function Qe(e, t, n) {
	return e + (t - e) * n;
}
function $e(e, t) {
	return Math.hypot(t.x - e.x, t.y - e.y);
}
function et(e, t) {
	let n = 1 - t;
	return {
		x: n * n * e.from.x + 2 * n * t * e.control.x + t * t * e.to.x,
		y: n * n * e.from.y + 2 * n * t * e.control.y + t * t * e.to.y
	};
}
function tt(e, t) {
	let n = 1 - t;
	return {
		x: n * n * n * e.from.x + 3 * n * n * t * e.control1.x + 3 * n * t * t * e.control2.x + t * t * t * e.to.x,
		y: n * n * n * e.from.y + 3 * n * n * t * e.control1.y + 3 * n * t * t * e.control2.y + t * t * t * e.to.y
	};
}
function nt(e) {
	let t = $e(e.from, e.to);
	if (t === 0) return;
	let n = Math.max(e.radius, t / 2), r = (e.from.x - e.to.x) / 2, i = (e.from.y - e.to.y) / 2, a = n * n, o = a * a - a * i * i - a * r * r, s = a * i * i + a * r * r, c = (e.largeArc === e.sweep ? -1 : 1) * Math.sqrt(Math.max(0, s === 0 ? 0 : o / s)), l = c * (n * i / n), u = c * -(n * r / n), d = {
		x: l + (e.from.x + e.to.x) / 2,
		y: u + (e.from.y + e.to.y) / 2
	}, f = Math.atan2((i - u) / n, (r - l) / n), p = Math.atan2((-i - u) / n, (-r - l) / n) - f;
	return e.sweep === 0 && p > 0 ? p -= Math.PI * 2 : e.sweep === 1 && p < 0 && (p += Math.PI * 2), {
		center: d,
		startAngle: f,
		sweepAngle: p
	};
}
function rt(e, t, n) {
	if (t === void 0) return {
		x: Qe(e.from.x, e.to.x, n),
		y: Qe(e.from.y, e.to.y, n)
	};
	let r = Math.max(e.radius, $e(e.from, e.to) / 2), i = t.startAngle + t.sweepAngle * n;
	return {
		x: t.center.x + Math.cos(i) * r,
		y: t.center.y + Math.sin(i) * r
	};
}
function it(e, t, n) {
	switch (e.kind) {
		case "line": return {
			x: Qe(e.from.x, e.to.x, t),
			y: Qe(e.from.y, e.to.y, t)
		};
		case "quad": return et(e, t);
		case "cubic": return tt(e, t);
		case "arc": return rt(e, n, t);
	}
}
var at = class {
	segments;
	length;
	#e;
	constructor(e) {
		this.segments = e, this.#e = e.map((e) => {
			let t = e.kind === "line" ? 1 : Ze, n = e.kind === "arc" ? nt(e) : void 0, r = [], i = [0], a = 0;
			for (let o = 0; o <= t; o += 1) {
				let s = it(e, o / t, n);
				if (o > 0) {
					let e = r[o - 1];
					e !== void 0 && (a += $e(e, s)), i.push(a);
				}
				r.push(s);
			}
			return {
				segment: e,
				points: r,
				cumulative: i,
				length: a
			};
		}), this.length = this.#e.reduce((e, t) => e + t.length, 0);
	}
	get start() {
		let e = this.segments[0];
		return e === void 0 ? {
			x: 0,
			y: 0
		} : e.from;
	}
	get end() {
		let e = this.segments[this.segments.length - 1];
		return e === void 0 ? {
			x: 0,
			y: 0
		} : e.to;
	}
	pointAt(e) {
		let t = Math.min(1, Math.max(0, Number.isFinite(e) ? e : 0)) * this.length, n = 0;
		for (let e of this.#e) {
			if (t <= n + e.length + 1e-9 || e === this.#e[this.#e.length - 1]) return ot(e, Math.min(e.length, Math.max(0, t - n)));
			n += e.length;
		}
		let r = this.end;
		return {
			x: r.x,
			y: r.y,
			angle: 0
		};
	}
	bounds() {
		let e = Infinity, t = Infinity, n = -Infinity, r = -Infinity;
		for (let i of this.#e) for (let a of i.points) e = Math.min(e, a.x), t = Math.min(t, a.y), n = Math.max(n, a.x), r = Math.max(r, a.y);
		return Number.isFinite(e) ? {
			x: e,
			y: t,
			width: n - e,
			height: r - t
		} : {
			x: 0,
			y: 0,
			width: 0,
			height: 0
		};
	}
	toSvg(e = 3) {
		let t = (t) => st(t, e), n = [], r;
		for (let e of this.segments) {
			switch ((r === void 0 || $e(r, e.from) > 1e-6) && n.push(`M ${t(e.from.x)} ${t(e.from.y)}`), e.kind) {
				case "line":
					n.push(`L ${t(e.to.x)} ${t(e.to.y)}`);
					break;
				case "quad":
					n.push(`Q ${t(e.control.x)} ${t(e.control.y)} ${t(e.to.x)} ${t(e.to.y)}`);
					break;
				case "cubic":
					n.push(`C ${t(e.control1.x)} ${t(e.control1.y)} ${t(e.control2.x)} ${t(e.control2.y)} ${t(e.to.x)} ${t(e.to.y)}`);
					break;
				case "arc": {
					let r = Math.max(e.radius, $e(e.from, e.to) / 2);
					n.push(`A ${t(r)} ${t(r)} 0 ${e.largeArc} ${e.sweep} ${t(e.to.x)} ${t(e.to.y)}`);
					break;
				}
			}
			r = e.to;
		}
		return n.join(" ");
	}
};
function ot(e, t) {
	let n = e.points, r = e.cumulative;
	for (let e = 1; e < n.length; e += 1) {
		let i = r[e] ?? 0, a = r[e - 1] ?? 0, o = n[e - 1], s = n[e];
		if (o !== void 0 && s !== void 0 && (t <= i || e === n.length - 1)) {
			let e = i - a, n = e <= 1e-9 ? 0 : Math.min(1, Math.max(0, (t - a) / e));
			return {
				x: Qe(o.x, s.x, n),
				y: Qe(o.y, s.y, n),
				angle: Math.atan2(s.y - o.y, s.x - o.x)
			};
		}
	}
	let i = n[0] ?? {
		x: 0,
		y: 0
	};
	return {
		x: i.x,
		y: i.y,
		angle: 0
	};
}
function st(e, t) {
	let n = Number(e.toFixed(t));
	return Object.is(n, -0) ? "0" : String(n);
}
function ct(e, t) {
	let n = [];
	if (e.length < 2) return n;
	if (t <= 0 || e.length === 2) {
		for (let t = 1; t < e.length; t += 1) {
			let r = e[t - 1], i = e[t];
			r !== void 0 && i !== void 0 && $e(r, i) > 1e-9 && n.push({
				kind: "line",
				from: r,
				to: i
			});
		}
		return n;
	}
	let r = e[0];
	if (r === void 0) return n;
	for (let i = 1; i < e.length - 1; i += 1) {
		let a = e[i], o = e[i + 1];
		if (a === void 0 || o === void 0) continue;
		let s = $e(r, a), c = $e(a, o), l = Math.min(t, s / 2, c / 2);
		if (l <= 1e-6) {
			s > 1e-9 && n.push({
				kind: "line",
				from: r,
				to: a
			}), r = a;
			continue;
		}
		let u = {
			x: a.x + (r.x - a.x) / s * l,
			y: a.y + (r.y - a.y) / s * l
		}, d = {
			x: a.x + (o.x - a.x) / c * l,
			y: a.y + (o.y - a.y) / c * l
		};
		$e(r, u) > 1e-9 && n.push({
			kind: "line",
			from: r,
			to: u
		}), n.push({
			kind: "quad",
			from: u,
			control: a,
			to: d
		}), r = d;
	}
	let i = e[e.length - 1];
	return i !== void 0 && $e(r, i) > 1e-9 && n.push({
		kind: "line",
		from: r,
		to: i
	}), n;
}
function lt(e) {
	return {
		x: e.x + e.width / 2,
		y: e.y + e.height / 2
	};
}
function ut(e, t, n = 0) {
	return e.x + e.width > t.x + n && t.x + t.width > e.x + n && e.y + e.height > t.y + n && t.y + t.height > e.y + n;
}
function dt(e, t) {
	let n = lt(e), r = t.x - n.x, i = t.y - n.y;
	if (Math.abs(r) < 1e-9 && Math.abs(i) < 1e-9) return n;
	let a = e.width / 2, o = e.height / 2, s = Math.abs(r) < 1e-9 ? Infinity : a / Math.abs(r), c = Math.abs(i) < 1e-9 ? Infinity : o / Math.abs(i), l = Math.min(s, c);
	return {
		x: n.x + r * l,
		y: n.y + i * l
	};
}
//#endregion
//#region ../core/dist/scene.js
var ft = [
	"wide",
	"compact",
	"narrow"
];
function pt(e, t = {}) {
	return {
		type: "linear-gradient",
		stops: e,
		...t
	};
}
function mt(e, t = {}) {
	return {
		type: "radial-gradient",
		stops: e,
		...t
	};
}
function ht(e, t = {}) {
	return pt([{
		at: 0,
		color: e,
		opacity: t.from ?? 1
	}, {
		at: 1,
		color: e,
		opacity: t.to ?? 0
	}], {
		angle: t.angle ?? 90,
		...t.spread === void 0 ? {} : { spread: t.spread }
	});
}
var gt = new Set(ft);
function _t(e) {
	if (typeof e != "object" || !e || Array.isArray(e)) return !1;
	let t = Object.keys(e);
	return t.length > 0 && t.every((e) => gt.has(e));
}
function vt(e, t) {
	if (e === void 0) return;
	if (!_t(e)) return e;
	let n = t === "narrow" ? [
		"narrow",
		"compact",
		"wide"
	] : t === "compact" ? ["compact", "wide"] : ["wide"];
	for (let t of n) {
		let n = e[t];
		if (n !== void 0) return n;
	}
}
function R(e, t, n) {
	return vt(e, t) ?? n;
}
function yt(e, t) {
	let n = (e, r, i) => {
		if (t(e, r, i), e.type === "group") for (let t of e.children) n(t, e, i + 1);
	};
	n(e, void 0, 0);
}
function bt(e) {
	return typeof e == "string" ? e : e.node;
}
function xt(e, t, n) {
	e.length === 0 ? n.push({
		severity: "error",
		code: "empty-id",
		message: `${t} id must not be empty`
	}) : /^[A-Za-z0-9_.:-]+$/.test(e) || n.push({
		severity: "error",
		code: "invalid-id",
		message: `${t} id "${e}" may only contain letters, digits, "_", ".", ":" and "-"`
	});
}
function St(e) {
	let t = [], n = /* @__PURE__ */ new Set();
	e.schemaVersion !== 2 && t.push({
		severity: "error",
		code: "schema",
		message: "schemaVersion must be 2"
	}), xt(e.id, "scene", t), e.title.length === 0 && t.push({
		severity: "error",
		code: "empty-title",
		message: "scene title must not be empty"
	}), e.root.type !== "group" && t.push({
		severity: "error",
		code: "root",
		message: "scene root must be a group"
	}), yt(e.root, (e) => {
		if (xt(e.id, "node", t), n.has(e.id) && t.push({
			severity: "error",
			code: "duplicate-id",
			message: `duplicate node id: ${e.id}`,
			path: e.id
		}), n.add(e.id), e.type === "path" && (e.viewBox.width <= 0 || e.viewBox.height <= 0) && t.push({
			severity: "error",
			code: "path-viewbox",
			message: `path ${e.id} needs a positive viewBox`,
			path: e.id
		}), e.type === "legend") {
			let n = /* @__PURE__ */ new Set();
			for (let r of e.items) n.has(r.id) && t.push({
				severity: "error",
				code: "duplicate-id",
				message: `legend ${e.id} repeats item id ${r.id}`,
				path: e.id
			}), n.add(r.id);
		}
		e.interactive && !e.label && !e.description && t.push({
			severity: "warning",
			code: "unlabelled-interactive",
			message: `interactive node ${e.id} has no label or description`,
			path: e.id
		});
	});
	let r = /* @__PURE__ */ new Set();
	for (let i of e.edges ?? []) {
		xt(i.id, "edge", t), (r.has(i.id) || n.has(i.id)) && t.push({
			severity: "error",
			code: "duplicate-id",
			message: `duplicate scene id: ${i.id}`,
			path: i.id
		}), r.add(i.id);
		for (let [e, r] of [["source", i.from], ["target", i.to]]) {
			let a = bt(r);
			n.has(a) || t.push({
				severity: "error",
				code: "missing-node",
				message: `edge ${i.id} refers to missing ${e} node ${a}`,
				path: i.id
			});
		}
		i.curvature !== void 0 && (i.curvature < 0 || i.curvature > 1) && t.push({
			severity: "error",
			code: "curvature",
			message: `edge ${i.id} curvature must be between 0 and 1`,
			path: i.id
		}), i.width !== void 0 && !(i.width > 0) && t.push({
			severity: "error",
			code: "edge-width",
			message: `edge ${i.id} width must be positive`,
			path: i.id
		});
	}
	let i = /* @__PURE__ */ new Set();
	for (let n of e.controls ?? []) xt(n.id, "control", t), i.has(n.id) && t.push({
		severity: "error",
		code: "duplicate-id",
		message: `duplicate control id: ${n.id}`,
		path: n.id
	}), i.add(n.id), (n.kind ?? "event") === "event" && !n.event && t.push({
		severity: "error",
		code: "control-event",
		message: `control ${n.id} must name an event`,
		path: n.id
	}), e.machine === void 0 && t.push({
		severity: "error",
		code: "control-machine",
		message: `control ${n.id} requires a scene state machine`,
		path: n.id
	});
	let a = e.timeline;
	if (a !== void 0) {
		(!Number.isFinite(a.duration) || a.duration < 0) && t.push({
			severity: "error",
			code: "timeline-duration",
			message: "timeline duration must be finite and non-negative"
		});
		let e = /* @__PURE__ */ new Set();
		for (let i of a.tracks) e.has(i.id) && t.push({
			severity: "error",
			code: "duplicate-id",
			message: `duplicate timeline track id: ${i.id}`,
			path: i.id
		}), e.add(i.id), !n.has(i.target) && !r.has(i.target) && t.push({
			severity: "error",
			code: "missing-target",
			message: `timeline track ${i.id} targets missing scene id ${i.target}`,
			path: i.id
		});
	}
	return {
		ok: t.every((e) => e.severity !== "error"),
		diagnostics: t
	};
}
function Ct(e) {
	let t = St(e).diagnostics.filter((e) => e.severity === "error");
	if (t.length > 0) throw Error(`invalid scene ${e.id || "(unnamed)"}:\n${t.map((e) => `- ${e.message}`).join("\n")}`);
	return e;
}
//#endregion
//#region ../core/dist/text.js
var wt = .01, Tt = /mono|menlo|consolas|courier|sfmono|jetbrains|fira code|source code|ibm plex mono/i;
function Et(e) {
	let t = e.codePointAt(0) ?? 32;
	return e === " " ? .28 : /[iljI'|!.,:;`]/.test(e) ? .29 : /[ftrJ]/.test(e) ? .37 : /[mwMW]/.test(e) ? .88 : /[a-z]/.test(e) ? .56 : /[A-Z]/.test(e) ? .69 : /[0-9]/.test(e) ? .58 : /[-()[\]{}/\\"*]/.test(e) ? .4 : /[+=<>#$_~^?]/.test(e) ? .58 : /[@%&]/.test(e) ? .86 : t >= 11904 ? 1 : t >= 128 ? .7 : .6;
}
function Dt(e) {
	return Tt.test(e);
}
function Ot(e, t) {
	if (e.length === 0) return 0;
	let n = Array.from(e), r = Dt(t.family), i = t.weight >= 600 ? 1.045 : 1, a = 0;
	for (let e of n) a += r ? .61 : Et(e) * 1.03;
	let o = (t.letterSpacing ?? 0) * Math.max(0, n.length - 1);
	return kt(a * t.size * i + o);
}
function kt(e) {
	return Math.round((e + 2 ** -52) * 1e3) / 1e3;
}
function At(e, t, n) {
	let r = Array.from(e), i = [], a = "";
	for (let e of r) {
		let r = a + e;
		a.length > 0 && Ot(r, n) > t + wt ? (i.push(a), a = e) : a = r;
	}
	return a.length > 0 && i.push(a), i;
}
function jt(e, t, n, r = {}) {
	let i = Math.max(1, t), a = Math.max(1, Math.floor(r.maxLines ?? Infinity)), o = e.trim().split(/\s+/).filter(Boolean), s = [], c = "", l = !1, u = () => {
		c.length > 0 && s.push(c), c = "";
	};
	outer: for (let e of o) {
		let t = Ot(e, n) > i + wt ? At(e, i, n) : [e];
		for (let e of t) {
			let t = c.length === 0 ? e : `${c} ${e}`;
			if (Ot(t, n) <= i + wt) {
				c = t;
				continue;
			}
			if (u(), s.length >= a) {
				l = !0;
				break outer;
			}
			c = e;
		}
	}
	if (!l && c.length > 0 && (s.length >= a ? l = !0 : u()), s.length === 0 && o.length === 0 && s.push(""), l && r.ellipsis !== !1 && s.length > 0) {
		let e = s.length - 1, t = s[e] ?? "";
		for (; t.length > 0 && Ot(`${t}…`, n) > i;) t = t.slice(0, -1).trimEnd();
		s[e] = `${t}…`;
	}
	return s.map((e) => ({
		text: e,
		width: Math.min(i, Ot(e, n))
	}));
}
//#endregion
//#region ../core/dist/theme.js
var Mt = {
	name: "default",
	colors: {
		canvas: "#f7f8fa",
		surface: "#ffffff",
		surfaceRaised: "#ffffff",
		surfaceMuted: "#eef0f4",
		text: "#15171a",
		textMuted: "#626973",
		accent: "#5b5ce2",
		accentContrast: "#ffffff",
		info: "#2f7bd9",
		success: "#16835d",
		warning: "#b26200",
		danger: "#c9363e",
		connector: "#969da8",
		border: "#dfe2e7",
		chart1: "#5b5ce2",
		chart2: "#2f7bd9",
		chart3: "#b26200",
		chart4: "#16835d",
		chart5: "#c9363e",
		chart6: "#7a8290",
		chartPositive: "#16835d",
		chartNegative: "#c9363e",
		chartNeutral: "#969da8"
	},
	spacing: {
		none: 0,
		xs: 4,
		sm: 8,
		md: 16,
		lg: 24,
		xl: 32,
		"2xl": 48
	},
	radii: {
		none: 0,
		sm: 4,
		md: 8,
		lg: 16,
		pill: 9999
	},
	typography: {
		label: {
			family: "Inter, sans-serif",
			size: 12,
			lineHeight: 16,
			weight: 600,
			letterSpacing: .2
		},
		caption: {
			family: "Inter, sans-serif",
			size: 12,
			lineHeight: 16,
			weight: 400
		},
		body: {
			family: "Inter, sans-serif",
			size: 16,
			lineHeight: 24,
			weight: 400
		},
		bodyStrong: {
			family: "Inter, sans-serif",
			size: 16,
			lineHeight: 24,
			weight: 600
		},
		title: {
			family: "Inter, sans-serif",
			size: 24,
			lineHeight: 30,
			weight: 650,
			letterSpacing: -.2
		},
		display: {
			family: "Inter, sans-serif",
			size: 44,
			lineHeight: 48,
			weight: 700,
			letterSpacing: -.8
		},
		code: {
			family: "ui-monospace, monospace",
			size: 14,
			lineHeight: 20,
			weight: 450
		}
	},
	motion: {
		fast: 120,
		normal: 240,
		slow: 480,
		easing: "easeOut"
	},
	strokes: {
		hairline: 1,
		thin: 1.5,
		regular: 2,
		bold: 3
	},
	ornament: {
		grid: "none",
		surface: "outlined",
		lineCap: "round",
		eyebrow: !0
	},
	materials: {
		flat: {},
		raised: {
			fill: "surfaceRaised",
			stroke: "border"
		},
		floating: {
			fill: "surfaceRaised",
			stroke: "border"
		},
		inset: {
			fill: "surfaceMuted",
			stroke: "border"
		},
		glass: {
			fill: {
				type: "linear-gradient",
				angle: 120,
				stops: [{
					at: 0,
					color: "surfaceRaised",
					opacity: .78
				}, {
					at: 1,
					color: "surface",
					opacity: .42
				}]
			},
			stroke: "border",
			effects: [{
				type: "backdrop",
				blur: 12,
				saturation: 1.08
			}]
		}
	}
};
function Nt(e = {}, t = Mt) {
	return {
		...e.name === void 0 ? t.name === void 0 ? {} : { name: t.name } : { name: e.name },
		colors: {
			...t.colors,
			...e.colors
		},
		spacing: {
			...t.spacing,
			...e.spacing
		},
		radii: {
			...t.radii,
			...e.radii
		},
		typography: {
			...t.typography,
			...e.typography
		},
		motion: {
			...t.motion,
			...e.motion
		},
		strokes: {
			...t.strokes ?? Mt.strokes,
			...e.strokes
		},
		ornament: {
			...t.ornament ?? Mt.ornament,
			...e.ornament
		},
		materials: {
			...t.materials ?? Mt.materials,
			...e.materials
		}
	};
}
var Pt = /* @__PURE__ */ new Set([
	"neutral",
	"accent",
	"success",
	"warning",
	"danger",
	"info",
	"muted"
]);
function Ft(e) {
	return Pt.has(e);
}
function It(e, t, n = "stroke") {
	switch (e) {
		case "accent": return t.colors.accent;
		case "success": return t.colors.success;
		case "warning": return t.colors.warning;
		case "danger": return t.colors.danger;
		case "info": return t.colors.info;
		case "muted": return n === "text" ? t.colors.textMuted : t.colors.border;
		case "neutral": return n === "text" ? t.colors.text : n === "fill" ? t.colors.surface : t.colors.border;
	}
}
function Lt(e, t, n, r) {
	if (e === void 0) return r;
	if (e === "none") return "none";
	if (Ft(e)) return It(e, t, n);
	let i = t.colors[e];
	return typeof i == "string" ? i : r;
}
function Rt(e, t, n) {
	let r = e.map((e) => ({
		at: Math.min(1, Math.max(0, Number.isFinite(e.at) ? e.at : 0)),
		color: Lt(e.color, t, "fill", n),
		opacity: Math.min(1, Math.max(0, Number.isFinite(e.opacity) ? e.opacity ?? 1 : 1))
	})).sort((e, t) => e.at - t.at);
	return r.length > 0 ? r : [{
		at: 0,
		color: n,
		opacity: 1
	}, {
		at: 1,
		color: n,
		opacity: 1
	}];
}
function zt(e, t, n) {
	if (e === void 0 || typeof e == "string") return Lt(e, t, "fill", n);
	if (e.type === "linear-gradient") return {
		type: e.type,
		stops: Rt(e.stops, t, n),
		angle: Number.isFinite(e.angle) ? e.angle ?? 0 : 0,
		spread: e.spread ?? "pad"
	};
	let r = e.center ?? [.5, .5], i = e.focalPoint ?? r;
	return {
		type: e.type,
		stops: Rt(e.stops, t, n),
		center: [Math.min(1, Math.max(0, r[0])), Math.min(1, Math.max(0, r[1]))],
		focalPoint: [Math.min(1, Math.max(0, i[0])), Math.min(1, Math.max(0, i[1]))],
		radius: Math.max(0, Number.isFinite(e.radius) ? e.radius ?? .5 : .5),
		spread: e.spread ?? "pad"
	};
}
function Bt(e, t) {
	return e !== void 0 && Number.isFinite(e) ? e : t;
}
function Vt(e, t, n) {
	return Math.min(n, Math.max(t, e));
}
function Ht(e, t) {
	switch (e.type) {
		case "shadow": return {
			type: e.type,
			kind: e.kind ?? "outer",
			color: Lt(e.color ?? "text", t, "fill", t.colors.text),
			opacity: Vt(Bt(e.opacity, .16), 0, 1),
			blur: Math.max(0, Bt(e.blur, 16)),
			spread: Bt(e.spread, 0),
			offset: [Bt(e.offset?.[0], 0), Bt(e.offset?.[1], 8)]
		};
		case "blur": return {
			type: e.type,
			radius: Math.max(0, Bt(e.radius, 0))
		};
		case "backdrop": return {
			type: e.type,
			blur: Math.max(0, Bt(e.blur, 16)),
			saturation: Math.max(0, Bt(e.saturation, 1)),
			brightness: Math.max(0, Bt(e.brightness, 1))
		};
		case "noise": return {
			type: e.type,
			amount: Vt(Bt(e.amount, .03), 0, 1),
			scale: Math.max(.01, Bt(e.scale, .8)),
			seed: Math.round(Bt(e.seed, 1)),
			monochrome: e.monochrome ?? !0
		};
	}
}
function Ut(e) {
	if (e.type !== "shader") return [];
	switch (e.name) {
		case "frosted-glass": return [{
			type: "backdrop",
			blur: 18,
			saturation: 1.16
		}, {
			type: "noise",
			amount: .025,
			scale: .7,
			seed: 17
		}];
		case "iridescence": return [{
			type: "noise",
			amount: .08,
			scale: .42,
			seed: 29,
			monochrome: !1
		}];
		case "liquid": return [{
			type: "blur",
			radius: .7
		}, {
			type: "noise",
			amount: .045,
			scale: .3,
			seed: 41
		}];
		case "grain": return [{
			type: "noise",
			amount: .035,
			scale: .9,
			seed: 7
		}];
	}
}
function Wt(e, t) {
	if (e.type !== "shader") return Ht(e, t);
	let n = e.fallback ?? Ut(e);
	return {
		type: e.type,
		name: e.name,
		uniforms: e.uniforms ?? {},
		fallback: n.map((e) => Ht(e, t))
	};
}
function Gt(e, t) {
	if (e === void 0) return {};
	let n = typeof e == "string" ? {} : e, r = typeof e == "string" ? e : e.material, i = r === void 0 ? {} : t.materials[r], a = n.effects ?? i.effects, o = {
		...i,
		...n,
		...a === void 0 ? {} : { effects: a }
	};
	return {
		...o.fill === void 0 ? {} : { fill: zt(o.fill, t, t.colors.surface) },
		...o.stroke === void 0 ? {} : { stroke: Lt(o.stroke, t, "stroke", "none") },
		...o.strokeWidth === void 0 ? {} : { strokeWidth: Math.max(0, Bt(o.strokeWidth, 0)) },
		...o.radius === void 0 ? {} : { radius: Math.max(0, Bt(o.radius, 0)) },
		...o.opacity === void 0 ? {} : { opacity: Vt(Bt(o.opacity, 1), 0, 1) },
		...o.effects === void 0 ? {} : { effects: o.effects.map((e) => Wt(e, t)) },
		...o.blendMode === void 0 ? {} : { blendMode: o.blendMode }
	};
}
function Kt(e, t, n) {
	let r = Math.min(1, Math.max(0, n));
	if (r <= 0) return e;
	if (r >= 1) return t;
	let i = Jt(e), a = Jt(t);
	if (i === void 0 || a === void 0) return r < .5 ? e : t;
	let o = (e) => Math.round((i[e] ?? 0) + ((a[e] ?? 0) - (i[e] ?? 0)) * r).toString(16).padStart(2, "0");
	return `#${o(0)}${o(1)}${o(2)}`;
}
function qt(e, t) {
	let n = Jt(e);
	if (n === void 0) return e;
	let r = Math.round(Math.min(1, Math.max(0, t)) * 255).toString(16).padStart(2, "0");
	return `#${n.slice(0, 3).map((e) => e.toString(16).padStart(2, "0")).join("")}${r}`;
}
function Jt(e) {
	let t = /^#([0-9a-f]{3}|[0-9a-f]{6}|[0-9a-f]{8})$/i.exec(e.trim());
	if (t === null) return;
	let n = t[1] ?? "";
	if (n.length === 3) {
		let [e, t, r] = n.split("");
		return [
			parseInt(`${e}${e}`, 16),
			parseInt(`${t}${t}`, 16),
			parseInt(`${r}${r}`, 16)
		];
	}
	return [
		parseInt(n.slice(0, 2), 16),
		parseInt(n.slice(2, 4), 16),
		parseInt(n.slice(4, 6), 16)
	];
}
function Yt(e) {
	return {
		background: e.colors.canvas,
		foreground: e.colors.text,
		accent: e.colors.accent,
		fontFamily: e.typography.body.family,
		semantic: {
			background: e.colors.canvas,
			surface: e.colors.surface,
			foreground: e.colors.text,
			muted: e.colors.connector,
			accent: e.colors.accent
		},
		node: {
			fill: e.colors.surface,
			stroke: e.colors.border,
			strokeWidth: 1,
			radius: e.radii.lg
		},
		edge: {
			stroke: e.colors.connector,
			strokeWidth: 2
		},
		text: {
			color: e.colors.text,
			fontFamily: e.typography.body.family,
			fontSize: e.typography.body.size
		},
		tokens: e
	};
}
//#endregion
//#region ../core/dist/edges.js
function Xt(e) {
	return typeof e == "string" ? { node: e } : e;
}
function Zt(e, t, n) {
	let r = lt(e), i = lt(t), a = i.x - r.x, o = i.y - r.y, s = Math.min(e.x + e.width, t.x + t.width) - Math.max(e.x, t.x), c = s > Math.min(e.width, t.width) * .35 && Math.abs(o) > .5;
	return n === "straight" && !c && Math.abs(a) < Math.abs(o) * .25 || c || Math.abs(o) > Math.abs(a) && s > 0 ? o >= 0 ? {
		from: "bottom",
		to: "top"
	} : {
		from: "top",
		to: "bottom"
	} : a >= 0 ? {
		from: "right",
		to: "left"
	} : {
		from: "left",
		to: "right"
	};
}
function Qt(e, t, n) {
	let r = [], i = /* @__PURE__ */ new Map();
	for (let a of e) {
		let e = Xt(a.from), o = Xt(a.to), s = n.get(e.node), c = n.get(o.node);
		if (s === void 0 || c === void 0) continue;
		let l = Zt(s, c, R(a.route, t, "straight")), u = R(e.side, t, "auto"), d = R(o.side, t, "auto"), f = {
			from: u === "auto" ? l.from : u,
			to: d === "auto" ? l.to : d
		};
		i.set(a.id, f), r.push({
			edgeId: a.id,
			end: "from",
			node: e.node,
			side: f.from,
			explicitOffset: vt(e.offset, t),
			otherCenter: lt(c)
		}), r.push({
			edgeId: a.id,
			end: "to",
			node: o.node,
			side: f.to,
			explicitOffset: vt(o.offset, t),
			otherCenter: lt(s)
		});
	}
	let a = /* @__PURE__ */ new Map();
	for (let e of r) {
		if (e.explicitOffset !== void 0) continue;
		let t = `${e.node}::${e.side}`, n = a.get(t) ?? [];
		n.push(e), a.set(t, n);
	}
	let o = /* @__PURE__ */ new Map();
	for (let [, e] of a) {
		let t = e[0]?.side === "top" || e[0]?.side === "bottom", n = [...e].sort((e, n) => {
			let r = t ? e.otherCenter.x - n.otherCenter.x : e.otherCenter.y - n.otherCenter.y;
			return Math.abs(r) > 1e-6 ? r : e.edgeId < n.edgeId ? -1 : e.edgeId > n.edgeId ? 1 : e.end === "from" ? -1 : 1;
		});
		n.forEach((e, t) => {
			o.set(`${e.edgeId}::${e.end}`, (t + 1) / (n.length + 1));
		});
	}
	let s = /* @__PURE__ */ new Map();
	for (let n of e) {
		let e = i.get(n.id);
		if (e === void 0) continue;
		let r = Xt(n.from), a = Xt(n.to);
		s.set(n.id, {
			from: {
				side: e.from,
				offset: vt(r.offset, t) ?? o.get(`${n.id}::from`) ?? .5
			},
			to: {
				side: e.to,
				offset: vt(a.offset, t) ?? o.get(`${n.id}::to`) ?? .5
			}
		});
	}
	return s;
}
function $t(e) {
	switch (e) {
		case "left": return {
			x: -1,
			y: 0
		};
		case "right": return {
			x: 1,
			y: 0
		};
		case "top": return {
			x: 0,
			y: -1
		};
		case "bottom": return {
			x: 0,
			y: 1
		};
		case "center": return {
			x: 0,
			y: 0
		};
	}
}
function en(e, t, n, r, i) {
	let a = Math.min(1, Math.max(0, n));
	if (t === "center") {
		if (e.kind === "circle" || e.kind === "ellipse") {
			let t = lt(e), n = i.x - t.x, a = i.y - t.y, o = Math.hypot(n, a) || 1, s = e.width / 2 + r, c = e.height / 2 + r;
			return {
				x: t.x + n / o * s,
				y: t.y + a / o * c
			};
		}
		let t = dt(e, i), n = lt(e), a = t.x - n.x, o = t.y - n.y, s = Math.hypot(a, o) || 1;
		return {
			x: t.x + a / s * r,
			y: t.y + o / s * r
		};
	}
	let o = $t(t);
	switch (t) {
		case "left": return {
			x: e.x - r,
			y: e.y + e.height * a
		};
		case "right": return {
			x: e.x + e.width + r,
			y: e.y + e.height * a
		};
		case "top": return {
			x: e.x + e.width * a,
			y: e.y - r
		};
		case "bottom": return {
			x: e.x + e.width * a,
			y: e.y + e.height + r
		};
		default: return {
			x: e.x + o.x,
			y: e.y + o.y
		};
	}
}
function tn(e, t, n, r) {
	let i = $t(t), a = $t(r), o = i.x !== 0, s = a.x !== 0, c = {
		x: e.x + i.x * 14,
		y: e.y + i.y * 14
	}, l = {
		x: n.x + a.x * 14,
		y: n.y + a.y * 14
	};
	if (t === "center" || r === "center") return [
		e,
		{
			x: n.x,
			y: e.y
		},
		n
	];
	if (o && s) {
		if (i.x !== a.x) {
			if (i.x > 0 ? l.x >= c.x : l.x <= c.x) {
				let t = (c.x + l.x) / 2;
				return nn([
					e,
					{
						x: t,
						y: e.y
					},
					{
						x: t,
						y: n.y
					},
					n
				]);
			}
			let t = (e.y + n.y) / 2;
			return nn([
				e,
				c,
				{
					x: c.x,
					y: t
				},
				{
					x: l.x,
					y: t
				},
				l,
				n
			]);
		}
		let t = i.x > 0 ? Math.max(c.x, l.x) : Math.min(c.x, l.x);
		return nn([
			e,
			{
				x: t,
				y: e.y
			},
			{
				x: t,
				y: n.y
			},
			n
		]);
	}
	if (!o && !s) {
		if (i.y !== a.y) {
			if (i.y > 0 ? l.y >= c.y : l.y <= c.y) {
				let t = (c.y + l.y) / 2;
				return nn([
					e,
					{
						x: e.x,
						y: t
					},
					{
						x: n.x,
						y: t
					},
					n
				]);
			}
			let t = (e.x + n.x) / 2;
			return nn([
				e,
				c,
				{
					x: t,
					y: c.y
				},
				{
					x: t,
					y: l.y
				},
				l,
				n
			]);
		}
		let t = i.y > 0 ? Math.max(c.y, l.y) : Math.min(c.y, l.y);
		return nn([
			e,
			{
				x: e.x,
				y: t
			},
			{
				x: n.x,
				y: t
			},
			n
		]);
	}
	if (o) {
		let t = {
			x: n.x,
			y: e.y
		}, r = i.x > 0 ? t.x >= c.x : t.x <= c.x, o = a.y > 0 ? t.y >= l.y : t.y <= l.y;
		return nn(r && o ? [
			e,
			t,
			n
		] : [
			e,
			c,
			{
				x: c.x,
				y: l.y
			},
			l,
			n
		]);
	}
	let u = {
		x: e.x,
		y: n.y
	}, d = i.y > 0 ? u.y >= c.y : u.y <= c.y, f = a.x > 0 ? u.x >= l.x : u.x <= l.x;
	return nn(d && f ? [
		e,
		u,
		n
	] : [
		e,
		c,
		{
			x: l.x,
			y: c.y
		},
		l,
		n
	]);
}
function nn(e) {
	let t = [];
	for (let n of e) {
		let e = t[t.length - 1];
		(e === void 0 || Math.abs(e.x - n.x) > 1e-6 || Math.abs(e.y - n.y) > 1e-6) && t.push(n);
	}
	return t;
}
function rn(e, t, n, r, i) {
	let a = $t(t), o = $t(r), s = Math.hypot(n.x - e.x, n.y - e.y), c = Math.min(260, Math.max(12, s * (.2 + i * .6))), l = {
		x: Math.sign(n.x - e.x) || 1,
		y: 0
	}, u = {
		x: -(Math.sign(n.x - e.x) || 1),
		y: 0
	}, d = t === "center" ? l : a, f = r === "center" ? u : o;
	return [{
		kind: "cubic",
		from: e,
		control1: {
			x: e.x + d.x * c,
			y: e.y + d.y * c
		},
		control2: {
			x: n.x + f.x * c,
			y: n.y + f.y * c
		},
		to: n
	}];
}
function an(e, t, n) {
	let r = Math.hypot(t.x - e.x, t.y - e.y), i = Math.min(Math.abs(n), r / 2);
	return r < 1e-6 || i < .5 ? [{
		kind: "line",
		from: e,
		to: t
	}] : [{
		kind: "arc",
		from: e,
		to: t,
		radius: r * r / (8 * i) + i / 2,
		sweep: n >= 0 ? 0 : 1,
		largeArc: 0
	}];
}
var on = {
	start: .14,
	middle: .5,
	end: .86
};
function sn(e, t, n, r, i) {
	let a = -Math.sin(t), o = Math.cos(t), s = e.x + a * n, c = e.y + o * n;
	return {
		x: s - r / 2,
		y: c - i / 2,
		width: r,
		height: i
	};
}
function cn(e, t, n) {
	let r = Xt(e.from), i = Xt(e.to), a = n.boxes.get(r.node), o = n.boxes.get(i.node);
	if (a === void 0 || o === void 0) return;
	let s = R(e.route, n.layout, "straight"), c = en(a, t.from.side, t.from.offset, r.gap ?? 0, lt(o)), l = en(o, t.to.side, t.to.offset, i.gap ?? 0, lt(a)), u = e.curvature ?? .5, d;
	switch (s) {
		case "straight":
			d = [{
				kind: "line",
				from: c,
				to: l
			}];
			break;
		case "orthogonal":
			d = ct(tn(c, t.from.side, l, t.to.side), e.cornerRadius ?? 8);
			break;
		case "curve":
			d = rn(c, t.from.side, l, t.to.side, u);
			break;
		case "arc": {
			let t = Math.hypot(l.x - c.x, l.y - c.y);
			d = an(c, l, e.bend ?? -t * .22 * (u / .5));
			break;
		}
	}
	let f = new at(d), p = n.theme, m = n.overrides?.tone ?? e.tone ?? "neutral", h = Lt(m, p, "stroke", p.colors.connector), g = m === "neutral" ? p.colors.connector : h, _ = e.width ?? p.strokes.regular, v = e.head ?? "arrow", y = e.tail ?? "none", b = e.stroke ?? "solid", x = [...e.label === void 0 ? [] : [{
		id: `${e.id}-label`,
		text: e.label,
		placement: "middle"
	}], ...(e.labels ?? []).map((t, n) => ({
		...t,
		id: t.id ?? `${e.id}-label-${n + 1}`
	}))], S = [], C = [], w = [];
	for (let e of x) {
		let t = n.overrides?.labelText?.get(e.id) ?? e.text, r = (n.overrides?.labelHidden?.has(e.id) ?? !1) || "hidden" in e && R(e.hidden, n.layout, !1), i = n.labelFont, a = Ot(t, i) + 10, o = i.lineHeight + 4, s = "placement" in e && e.placement !== void 0 ? e.placement : "middle", c = f.pointAt(on[s]), l = "offset" in e && e.offset !== void 0 ? e.offset : -(o / 2 + 4), u = [];
		for (let e = 1; e <= 6; e += 1) u.push(l * e, -l * e);
		let d = n.bounds, m = (e) => d === void 0 || e.x >= d.x - .5 && e.y >= d.y - .5 && e.x + e.width <= d.x + d.width + .5 && e.y + e.height <= d.y + d.height + .5, h = sn(c, c.angle, l, a, o);
		for (let e of u) {
			let t = sn(c, c.angle, e, a, o);
			if (!(n.obstacles.some((e) => ut(t, e, 1)) || C.some((e) => ut(t, e, 1))) && m(t)) {
				h = t;
				break;
			}
		}
		if (d !== void 0 && !m(h)) {
			let e = Math.min(Math.max(h.x, d.x), d.x + d.width - h.width), t = Math.min(Math.max(h.y, d.y), d.y + d.height - h.height);
			h = {
				...h,
				x: e,
				y: t
			};
		}
		!r && n.obstacles.some((e) => ut(h, e, 1)) && w.push(e.id), C.push(h);
		let g = "tone" in e && e.tone !== void 0 ? Lt(e.tone, p, "text", n.labelColor) : n.labelColor;
		S.push({
			id: e.id,
			text: t,
			x: fn(h.x + h.width / 2, n.precision),
			y: fn(h.y + h.height / 2, n.precision),
			width: fn(h.width, n.precision),
			height: fn(h.height, n.precision),
			anchor: "middle",
			fontFamily: i.family,
			fontSize: i.size,
			fontWeight: i.weight,
			color: g,
			...r ? { hidden: !0 } : {}
		});
	}
	let T = e.packets, E = T === void 0 ? 0 : Math.max(1, Math.floor(T.count ?? 2)), D = T?.period ?? 2400, O = Lt(T?.tone ?? m, p, "stroke", g), k = x[0]?.text;
	return {
		edge: {
			id: e.id,
			from: r.node,
			to: i.node,
			start: pn(c, n.precision),
			end: pn(l, n.precision),
			path: f.toSvg(n.precision),
			directed: v !== "none",
			...k === void 0 ? {} : { label: n.overrides?.label ?? k },
			appearance: {
				stroke: g,
				strokeWidth: _,
				...e.opacity === void 0 ? {} : { opacity: e.opacity }
			},
			state: {
				opacity: 1,
				progress: 1,
				highlight: 0,
				flow: T === void 0 ? 0 : 1
			},
			route: s,
			head: v,
			tail: y,
			dash: b,
			length: fn(f.length, n.precision),
			samples: ln(f, n.precision),
			labels: S,
			packets: [],
			...T === void 0 ? {} : {
				packetSize: T.size ?? Math.max(3, _ * 1.6),
				packetColor: O
			},
			...e.description === void 0 ? {} : { description: e.description },
			...e.z === void 0 ? {} : { z: e.z },
			...n.overrides?.hidden === !0 || vt(e.hidden, n.layout) === !0 ? { hidden: !0 } : {},
			...e.metadata === void 0 ? {} : { metadata: e.metadata }
		},
		geometry: f,
		packetCount: E,
		packetPeriod: D,
		collidingLabels: w
	};
}
function ln(e, t) {
	let n = [];
	for (let r = 0; r <= 32; r += 1) n.push(pn(e.pointAt(r / 32), t));
	return n;
}
function un(e, t) {
	if (e.length === 0) return {
		x: 0,
		y: 0
	};
	let n = Math.min(1, Math.max(0, Number.isFinite(t) ? t : 0)) * (e.length - 1), r = Math.min(e.length - 2, Math.floor(n)), i = e[Math.max(0, r)], a = e[Math.min(e.length - 1, r + 1)];
	if (i === void 0) return {
		x: 0,
		y: 0
	};
	if (a === void 0) return i;
	let o = n - r;
	return {
		x: i.x + (a.x - i.x) * o,
		y: i.y + (a.y - i.y) * o
	};
}
function dn(e, t, n, r, i = 3) {
	if (t <= 0 || n <= 0 || e.length === 0) return [];
	let a = (r % n + n) % n / n, o = [];
	for (let n = 0; n < t; n += 1) {
		let r = (a + n / t) % 1;
		o.push(pn(un(e, r), i));
	}
	return o;
}
function fn(e, t) {
	let n = 10 ** t;
	return Math.round((e + 2 ** -52) * n) / n;
}
function pn(e, t) {
	return {
		x: fn(e.x, t),
		y: fn(e.y, t)
	};
}
//#endregion
//#region ../core/dist/fragment.js
function mn(e, t) {
	return e.length === 0 || t.startsWith(`${e}:`) ? t : `${e}:${t}`;
}
function hn(e, t) {
	let n = {
		...e,
		id: mn(t, e.id)
	};
	return n.type === "group" ? {
		...n,
		children: n.children.map((e) => hn(e, t))
	} : n.type === "legend" ? {
		...n,
		items: n.items.map((e) => ({
			...e,
			id: mn(t, e.id)
		}))
	} : n;
}
function gn(e, t) {
	return t.length === 0 ? e : {
		...e,
		nodes: e.nodes.map((e) => hn(e, t)),
		...e.edges === void 0 ? {} : { edges: e.edges.map((e) => ({
			...e,
			id: mn(t, e.id),
			...e.labels === void 0 ? {} : { labels: e.labels.map((e) => e.id === void 0 ? e : {
				...e,
				id: mn(t, e.id)
			}) },
			from: typeof e.from == "string" ? mn(t, e.from) : {
				...e.from,
				node: mn(t, e.from.node)
			},
			to: typeof e.to == "string" ? mn(t, e.to) : {
				...e.to,
				node: mn(t, e.to.node)
			}
		})) },
		...e.tracks === void 0 ? {} : { tracks: e.tracks.map((e) => ({
			...e,
			id: mn(t, e.id),
			target: mn(t, e.target)
		})) },
		...e.controls === void 0 ? {} : { controls: e.controls.map((e) => ({
			...e,
			id: mn(t, e.id)
		})) },
		...e.diagnostics === void 0 ? {} : { diagnostics: e.diagnostics.map((e) => e.path === void 0 ? e : {
			...e,
			path: mn(t, e.path)
		}) }
	};
}
function _n(e, t) {
	return t === 0 ? e : e.map((e) => ({
		...e,
		keyframes: e.keyframes.map((e) => ({
			...e,
			time: e.time + t
		}))
	}));
}
function vn(e) {
	let t = 0;
	for (let n of e ?? []) for (let e of n.keyframes) t = Math.max(t, e.time);
	return t;
}
//#endregion
//#region ../core/dist/machine.js
function yn(e) {
	return typeof e == "string" ? [{ target: e }] : Array.isArray(e) ? e : [e];
}
function bn(e, t) {
	"var" in e ? t.push(e.var) : "all" in e ? e.all.forEach((e) => bn(e, t)) : "any" in e ? e.any.forEach((e) => bn(e, t)) : "not" in e && bn(e.not, t);
}
function xn(e, t) {
	"state" in e ? t.push(...typeof e.state == "string" ? [e.state] : e.state) : "all" in e ? e.all.forEach((e) => xn(e, t)) : "any" in e ? e.any.forEach((e) => xn(e, t)) : "not" in e && xn(e.not, t);
}
function Sn(e, t) {
	"selection" in e && e.selection !== null ? t.push(e.selection) : "all" in e ? e.all.forEach((e) => Sn(e, t)) : "any" in e ? e.any.forEach((e) => Sn(e, t)) : "not" in e && Sn(e.not, t);
}
function Cn(e) {
	return typeof e == "object" && !!e;
}
function wn(e, t) {
	if (Cn(e)) {
		if ("var" in e) t.vars.push(e.var);
		else if ("signal" in e) t.signals.push(e.signal);
		else if ("when" in e) t.conditions.push(e.when), wn(e.then, t), e.else !== void 0 && wn(e.else, t);
		else if ("match" in e) {
			wn(e.match, t);
			for (let n of Object.values(e.cases)) wn(n, t);
			e.default !== void 0 && wn(e.default, t);
		} else if ("concat" in e) for (let n of e.concat) wn(n, t);
		else "not" in e && wn(e.not, t);
	}
}
function Tn(e, t = {}) {
	let n = [], r = (e, t, r) => {
		n.push({
			severity: "error",
			code: e,
			message: t,
			...r === void 0 ? {} : { path: r }
		});
	}, i = (e, t, r) => {
		n.push({
			severity: "warning",
			code: e,
			message: t,
			...r === void 0 ? {} : { path: r }
		});
	}, a = t.nodeIds === void 0 ? void 0 : t.nodeIds instanceof Set ? t.nodeIds : new Set(t.nodeIds), o = new Set(Object.keys(e.variables ?? {})), s = Object.keys(e.states), c = Object.keys(e.signals ?? {}), l = new Set(s);
	e.id.length === 0 && r("empty-id", "machine id must not be empty"), s.length === 0 && r("no-states", `machine ${e.id} declares no states`), l.has(e.initial) || r("unknown-initial", `machine ${e.id} initial state "${e.initial}" is not defined`);
	for (let e of o) e.length === 0 && r("empty-variable", "variable names must not be empty"), e.startsWith("$") && r("reserved-variable", `variable "${e}" uses the reserved "$" prefix`);
	for (let e of c) o.has(e) && r("signal-collision", `signal "${e}" collides with a variable of the same name`), e.startsWith("$") && r("reserved-signal", `signal "${e}" uses the reserved "$" prefix`);
	let u = (e, t) => {
		let n = [], i = [], s = [];
		bn(e, n), xn(e, i), Sn(e, s);
		for (let e of n) o.has(e) || r("unknown-variable", `${t} refers to unknown variable "${e}"`, t);
		for (let e of i) l.has(e) || r("unknown-state", `${t} refers to unknown state "${e}"`, t);
		if (a !== void 0) for (let e of s) a.has(e) || r("unknown-node", `${t} refers to unknown node "${e}"`, t);
	}, d = (e, t) => {
		(e ?? []).forEach((e, n) => {
			let i = `${t}.actions[${n}]`;
			e.type === "set" || e.type === "toggle" || e.type === "increment" ? (o.has(e.var) || r("unknown-variable", `${i} refers to unknown variable "${e.var}"`, i), e.type === "increment" && e.by !== void 0 && !Number.isFinite(e.by) && r("invalid-action", `${i} increment step must be finite`, i)) : e.type === "select" ? a !== void 0 && e.node !== null && !a.has(e.node) && r("unknown-node", `${i} selects unknown node "${e.node}"`, i) : e.type === "seek" ? typeof e.time == "number" && (!Number.isFinite(e.time) || e.time < 0) && r("invalid-action", `${i} seek time must be finite and non-negative`, i) : r("invalid-action", `${i} has an unknown action type`, i);
		});
	}, f = /* @__PURE__ */ new Set(), p = l.has(e.initial) ? [e.initial] : [];
	for (let [t, n] of Object.entries(e.states)) {
		let e = `states.${t}`;
		t.length === 0 && r("empty-id", "state ids must not be empty"), d(n.entry, `${e}.entry`), d(n.exit, `${e}.exit`);
		for (let [t, a] of Object.entries(n.on ?? {})) {
			t.length === 0 && r("empty-event", `${e} declares an empty event name`, e);
			let n = yn(a);
			n.length === 0 && r("empty-transition", `${e}.on.${t} declares no transitions`, e), n.forEach((a, o) => {
				let s = `${e}.on.${t}[${o}]`;
				l.has(a.target) || r("unknown-target", `${s} targets unknown state "${a.target}"`, s), a.guard !== void 0 && u(a.guard, `${s}.guard`), d(a.actions, s), o < n.length - 1 && a.guard === void 0 && i("unreachable-transition", `${s} has no guard, so later transitions for "${t}" can never fire`, s);
			});
		}
	}
	for (; p.length > 0;) {
		let t = p.shift();
		if (!(t === void 0 || f.has(t))) {
			f.add(t);
			for (let n of Object.values(e.states[t]?.on ?? {})) for (let e of yn(n)) l.has(e.target) && !f.has(e.target) && p.push(e.target);
		}
	}
	for (let t of s) !f.has(t) && l.has(e.initial) && i("unreachable-state", `state "${t}" is unreachable from "${e.initial}"`, t);
	let m = /* @__PURE__ */ new Set();
	for (let [t, n] of Object.entries(e.signals ?? {})) {
		let e = `signals.${t}`, i = {
			vars: [],
			signals: [],
			conditions: []
		};
		wn(n, i);
		for (let t of i.vars) o.has(t) || r("unknown-variable", `${e} refers to unknown variable "${t}"`, e);
		for (let n of i.signals) n === t ? r("signal-cycle", `${e} refers to itself`, e) : m.has(n) || r("signal-order", `${e} refers to signal "${n}" which is not declared earlier (signals evaluate in order)`, e);
		for (let t of i.conditions) u(t, e);
		m.add(t);
	}
	return {
		ok: n.every((e) => e.severity !== "error"),
		diagnostics: n
	};
}
function En(e) {
	return e != null && e !== !1 && e !== 0 && e !== "";
}
function Dn(e, t, n) {
	switch (t) {
		case "eq": return e === n;
		case "neq": return e !== n;
		case "gt": return typeof e == "number" && typeof n == "number" && e > n;
		case "gte": return typeof e == "number" && typeof n == "number" && e >= n;
		case "lt": return typeof e == "number" && typeof n == "number" && e < n;
		case "lte": return typeof e == "number" && typeof n == "number" && e <= n;
		case "in": return Array.isArray(n) && n.includes(e);
		case "truthy": return En(e);
		case "falsy": return !En(e);
	}
}
function On(e, t) {
	if ("var" in e) {
		let n = e.op ?? (e.value === void 0 ? "truthy" : "eq");
		return Dn(t.variables[e.var], n, e.value);
	}
	return "state" in e ? typeof e.state == "string" ? t.state === e.state : e.state.includes(t.state) : "selection" in e ? t.selection === e.selection : "all" in e ? e.all.every((e) => On(e, t)) : "any" in e ? e.any.some((e) => On(e, t)) : !On(e.not, t);
}
function kn(e) {
	return e == null ? "" : typeof e == "string" ? e : String(e);
}
function An(e, t, n) {
	if (!Cn(e)) return e;
	if ("var" in e) return t.variables[e.var] ?? null;
	if ("signal" in e) return n[e.signal] ?? null;
	if ("state" in e) return t.state;
	if ("selection" in e) return t.selection;
	if ("when" in e) return On(e.when, t) ? An(e.then, t, n) : e.else === void 0 ? null : An(e.else, t, n);
	if ("match" in e) {
		let r = kn(An(e.match, t, n)), i = Object.hasOwn(e.cases, r) ? e.cases[r] : void 0;
		return i === void 0 ? e.default === void 0 ? null : An(e.default, t, n) : An(i, t, n);
	}
	return "concat" in e ? e.concat.map((e) => kn(An(e, t, n))).join(e.separator ?? "") : !En(An(e.not, t, n));
}
function jn(e, t) {
	let n = { ...t.variables };
	n.$state = t.state, n.$selection = t.selection;
	for (let [r, i] of Object.entries(e.signals ?? {})) n[r] = An(i, t, n);
	return n;
}
function Mn(e, t, n, r) {
	let i, a = t.selection;
	for (let o of e ?? []) switch (o.type) {
		case "set": {
			i ??= { ...t.variables };
			let e = typeof o.value == "object" && o.value !== null ? n?.value ?? null : o.value;
			i[o.var] = e;
			break;
		}
		case "toggle":
			i ??= { ...t.variables }, i[o.var] = !En(i[o.var]);
			break;
		case "increment": {
			i ??= { ...t.variables };
			let e = i[o.var], n = (typeof e == "number" ? e : 0) + (o.by ?? 1);
			o.min !== void 0 && (n = Math.max(o.min, n)), o.max !== void 0 && (n = Math.min(o.max, n)), i[o.var] = n;
			break;
		}
		case "select":
			a = o.node;
			break;
		case "seek": r.push({
			type: "seek",
			time: o.time
		});
	}
	return {
		state: t.state,
		variables: i ?? t.variables,
		selection: a
	};
}
function Nn(e) {
	return Pn(e, e.initial);
}
function Pn(e, t, n = {}) {
	if (!Object.hasOwn(e.states, t)) throw Error(`machine ${e.id} has no state "${t}"`);
	let r = {
		state: t,
		variables: { ...e.variables ?? {} },
		selection: null
	}, i = Mn(e.states[t]?.entry, r, void 0, []);
	return {
		state: t,
		variables: {
			...i.variables,
			...n.variables ?? {}
		},
		selection: n.selection === void 0 ? i.selection : n.selection
	};
}
function Fn(e, t, n) {
	let r = typeof n == "string" ? { type: n } : n, i = e.states[t.state];
	if (i === void 0) throw Error(`machine ${e.id} is in unknown state "${t.state}"`);
	let a = i.on?.[r.type], o = [], s = {
		previous: t,
		next: t,
		event: r,
		changed: !1,
		effects: o
	};
	if (a === void 0) return s;
	let c = yn(a).find((e) => e.guard === void 0 || On(e.guard, t));
	if (c === void 0) return s;
	let l = e.states[c.target];
	if (l === void 0) throw Error(`machine ${e.id} transition targets unknown state "${c.target}"`);
	let u = Mn(i.exit, t, r, o);
	u = Mn(c.actions, u, r, o), u = {
		...u,
		state: c.target
	}, u = Mn(l.entry, u, r, o);
	let d = u.state !== t.state || u.selection !== t.selection || !In(u.variables, t.variables) || o.length > 0;
	return {
		previous: t,
		next: u,
		event: r,
		changed: d,
		transition: {
			from: t.state,
			to: c.target,
			event: r.type
		},
		effects: o
	};
}
function In(e, t) {
	let n = Object.keys(e);
	return n.length === Object.keys(t).length && n.every((n) => e[n] === t[n]);
}
var Ln = class {
	machine;
	#e;
	#t;
	#n;
	constructor(e, t = {}) {
		this.machine = e, this.#e = t.initialState ?? Nn(e), this.#t = t.history === !0 ? [] : void 0, this.#n = t.onChange;
	}
	get state() {
		return this.#e;
	}
	get signals() {
		return jn(this.machine, this.#e);
	}
	get history() {
		return this.#t ?? [];
	}
	send(e) {
		let t = Fn(this.machine, this.#e, e);
		return t.transition !== void 0 && (this.#e = t.next, this.#t?.push({
			event: t.event,
			from: t.transition.from,
			to: t.transition.to
		}), this.#n?.(t)), t;
	}
	reset() {
		let e = this.#e;
		this.#e = Nn(this.machine), this.#t?.splice(0, this.#t.length);
		let t = {
			previous: e,
			next: this.#e,
			event: { type: "$reset" },
			changed: !0,
			transition: {
				from: e.state,
				to: this.#e.state,
				event: "$reset"
			},
			effects: [{
				type: "seek",
				time: "start"
			}]
		};
		return this.#n?.(t), t;
	}
	select(e) {
		if (this.#e.selection === e) return;
		let t = this.#e;
		this.#e = {
			...this.#e,
			selection: e
		}, this.#n?.({
			previous: t,
			next: this.#e,
			event: {
				type: "$select",
				value: e
			},
			changed: !0,
			effects: []
		});
	}
};
//#endregion
//#region ../core/dist/recipes.js
function Rn(e, t, n, r = {}) {
	return {
		id: e,
		type: "text",
		text: t,
		...n === void 0 ? {} : { textStyle: n },
		...r.tone === void 0 ? {} : { color: r.tone },
		...r.align === void 0 ? {} : { align: r.align },
		...r.maxLines === void 0 ? {} : { maxLines: r.maxLines },
		...r.bind === void 0 ? {} : { bind: r.bind },
		...r.hidden === void 0 ? {} : { hidden: r.hidden },
		...r.width === void 0 ? {} : { width: r.width },
		...r.transform === void 0 ? {} : { transform: r.transform }
	};
}
var zn = (e, t, n = {}) => Rn(e, t, n.textStyle, n), Bn = (e, t, n) => Rn(e, t, "label", n), Vn = (e, t, n) => Rn(e, t, "bodyStrong", n), Hn = (e, t, n) => Rn(e, t, "title", n), Un = (e, t, n) => Rn(e, t, "caption", {
	maxLines: 4,
	...n
}), Wn = (e, t, n) => Rn(e, t, "code", n);
function Gn(e, t, n = {}) {
	return {
		id: e,
		type: "badge",
		text: t,
		tone: n.tone ?? "accent",
		variant: n.variant ?? "soft",
		...n.bind === void 0 ? {} : { bind: n.bind },
		...n.hidden === void 0 ? {} : { hidden: n.hidden }
	};
}
function Kn(e, t, n = {}) {
	return {
		id: e,
		type: "icon",
		icon: t,
		tone: n.tone ?? "accent",
		size: n.size ?? 24,
		...n.background === void 0 ? {} : { background: n.background }
	};
}
function qn(e, t, n, r = {}) {
	return {
		id: e,
		type: "group",
		layout: t,
		children: n,
		...r.gap === void 0 ? {} : { gap: r.gap },
		...r.padding === void 0 ? {} : { padding: r.padding },
		...r.align === void 0 ? {} : { align: r.align },
		...r.justify === void 0 ? {} : { justify: r.justify },
		...r.width === void 0 ? {} : { width: r.width },
		...r.height === void 0 ? {} : { height: r.height },
		...r.minWidth === void 0 ? {} : { minWidth: r.minWidth },
		...r.maxWidth === void 0 ? {} : { maxWidth: r.maxWidth },
		...r.grow === void 0 ? {} : { grow: r.grow },
		...r.columns === void 0 ? {} : { columns: r.columns },
		...r.frame === void 0 ? {} : { frame: r.frame },
		...r.hidden === void 0 ? {} : { hidden: r.hidden },
		...r.z === void 0 ? {} : { z: r.z },
		...r.label === void 0 ? {} : { label: r.label },
		...r.description === void 0 ? {} : { description: r.description },
		...r.interactive === void 0 ? {} : { interactive: r.interactive },
		...r.onActivate === void 0 ? {} : { onActivate: r.onActivate },
		...r.bind === void 0 ? {} : { bind: r.bind },
		...r.metadata === void 0 ? {} : { metadata: r.metadata },
		...r.alignSelf === void 0 ? {} : { alignSelf: r.alignSelf },
		...r.clip === void 0 ? {} : { clip: r.clip },
		...r.minHeight === void 0 ? {} : { minHeight: r.minHeight },
		...r.justifySelf === void 0 ? {} : { justifySelf: r.justifySelf },
		...r.position === void 0 ? {} : { position: r.position },
		...r.opacity === void 0 ? {} : { opacity: r.opacity },
		...r.focusGroup === void 0 ? {} : { focusGroup: r.focusGroup },
		...r.inspect === void 0 ? {} : { inspect: r.inspect },
		...r.revealAnchor === void 0 ? {} : { revealAnchor: r.revealAnchor },
		...r.allowOverflow === void 0 ? {} : { allowOverflow: r.allowOverflow }
	};
}
var z = (e, t, n) => qn(e, "stack", t, n), Jn = (e, t, n) => qn(e, "row", t, n), Yn = (e, t, n) => qn(e, {
	wide: "row",
	compact: "stack"
}, t, n);
function Xn(e, t) {
	let n = t.tone ?? "accent", r = [], i = [];
	t.eyebrow !== void 0 && i.push(Bn(`${e}-eyebrow`, t.eyebrow)), i.push(Vn(`${e}-title`, t.title, { ...t.titleBind === void 0 ? {} : { bind: t.titleBind } })), t.motif !== void 0 && (r.push(Kn(`${e}-motif`, t.motif, { tone: n })), r.push(z(`${e}-heading`, i, {
		gap: 2,
		width: "fill"
	})));
	let a = [];
	return r.length > 0 ? a.push(Jn(`${e}-header`, r, {
		gap: 12,
		align: "center",
		width: "fill"
	})) : a.push(...i), t.body !== void 0 && a.push(Un(`${e}-body`, t.body, { ...t.bodyBind === void 0 ? {} : { bind: t.bodyBind } })), t.badge !== void 0 && a.push(Gn(`${e}-badge`, t.badge, {
		tone: t.badgeTone ?? n,
		...t.badgeBind === void 0 ? {} : { bind: t.badgeBind }
	})), t.extras !== void 0 && a.push(...t.extras), z(e, a, {
		gap: t.compact ? 6 : 8,
		padding: t.compact ? [12, 14] : [16, 18],
		frame: {
			fill: "surface",
			stroke: "border"
		},
		width: "fill",
		label: t.label ?? t.title,
		...t.body === void 0 ? {} : { description: t.body },
		...Qn(t)
	});
}
var Zn = /* @__PURE__ */ "gap.padding.align.justify.width.height.minWidth.maxWidth.grow.columns.frame.hidden.z.label.description.interactive.onActivate.bind.metadata.alignSelf.clip.minHeight.justifySelf.position.opacity.focusGroup.inspect.revealAnchor.allowOverflow".split(".");
function Qn(e, t = []) {
	let n = {};
	for (let r of Zn) {
		if (t.includes(r)) continue;
		let i = e[r];
		i !== void 0 && (n[r] = i);
	}
	return n;
}
function $n(e, t, n = {}) {
	let r = [];
	n.eyebrow !== void 0 && r.push(Bn(`${e}-eyebrow`, n.eyebrow, n.tone === void 0 ? {} : { tone: n.tone })), n.title !== void 0 && r.push(Vn(`${e}-title`, n.title));
	let i = qn(`${e}-content`, n.layout ?? "stack", t, {
		gap: n.gap ?? 12,
		width: "fill",
		...n.columns === void 0 ? {} : { columns: n.columns }
	});
	return z(e, r.length > 0 ? [z(`${e}-head`, r, { gap: 2 }), i] : [i], {
		gap: 12,
		padding: 16,
		frame: {
			fill: "surfaceMuted",
			stroke: "border",
			dash: "dashed"
		},
		width: "fill",
		...Qn(n, ["columns", "gap"])
	});
}
function er(e, t = "border") {
	return {
		id: e,
		type: "rect",
		width: "fill",
		height: 1,
		fill: t,
		stroke: "none",
		radius: 0
	};
}
function tr(e, t) {
	return {
		id: e,
		type: "rect",
		width: 1,
		height: t,
		fill: "none",
		stroke: "none"
	};
}
function nr(e, t, n, r = {}) {
	return Jn(e, [Un(`${e}-key`, t), Wn(`${e}-value`, n, { tone: r.valueTone ?? "text" })], {
		gap: 8,
		justify: "between",
		width: "fill",
		align: "center"
	});
}
//#endregion
//#region ../core/dist/figure.js
var rr = 32, ir = /^[A-Za-z0-9_.:-]+$/, ar = {
	reveal: 500,
	draw: 450,
	pulse: 500,
	highlight: 500,
	progress: 600,
	rise: 500,
	wipe: 500,
	gap: 120
};
function or(e) {
	let t = e.normalize("NFKD").replace(/[\u0300-\u036f]/g, "").toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-+|-+$/g, "");
	return t.length <= rr ? t : t.slice(0, rr).replace(/-+$/g, "");
}
function sr(e, t = 28) {
	return e.length <= t ? e : `${e.slice(0, t - 1)}…`;
}
function cr(e, t) {
	return `f.${e}(${t === void 0 ? "…" : JSON.stringify(sr(t))})`;
}
function lr(e) {
	return typeof e == "object" && !!e && "type" in e && "id" in e && !("from" in e);
}
function ur(e) {
	return Array.isArray(e) && e.every(lr);
}
function dr(e) {
	return Array.isArray(e.nodes);
}
function fr(e) {
	return !dr(e) && "handles" in e;
}
function pr(e) {
	let t = [];
	for (let n of e.keyframes) {
		let e = t[t.length - 1];
		e !== void 0 && n.time <= e.time ? t[t.length - 1] = {
			...n,
			time: e.time
		} : t.push(n);
	}
	return t.length === e.keyframes.length ? e : {
		...e,
		keyframes: t
	};
}
function mr(e, t, n, r, i, a, o = "easeOut") {
	let s = [];
	return n > 0 && s.push({
		time: 0,
		value: i
	}), s.push({
		time: n,
		value: i
	}, {
		time: r,
		value: a,
		easing: o
	}), pr(Ae(e, t, s));
}
function hr(e, t) {
	return t === void 0 ? [...e] : e.map((e) => ({
		...e,
		keyframes: e.keyframes.map((e) => e.easing === void 0 ? e : {
			...e,
			easing: t
		})
	}));
}
function gr(e, t) {
	return e === t || e.startsWith(`${t}:`);
}
function _r(e, t) {
	let n = !0, r = (e) => {
		gr(e.id, t) || (n = !1), e.type === "group" && e.children.forEach(r);
	};
	e.nodes.forEach(r);
	for (let r of e.edges ?? []) (!gr(r.id, t) || !gr(bt(r.from), t) || !gr(bt(r.to), t)) && (n = !1);
	for (let r of e.tracks ?? []) (!gr(r.id, t) || !gr(r.target, t)) && (n = !1);
	for (let r of e.controls ?? []) gr(r.id, t) || (n = !1);
	return n;
}
function vr(e) {
	return e === void 0 ? [] : Object.entries(e).filter((e) => typeof e[1] == "string");
}
function yr(e, t) {
	let n = /* @__PURE__ */ new Map(), r = /* @__PURE__ */ new Set(), i = /* @__PURE__ */ new Set(), a = [], o = /* @__PURE__ */ new Set(), s = [], c = [], l = /* @__PURE__ */ new Set(), u = [], d = /* @__PURE__ */ new Set(), f = /* @__PURE__ */ new WeakMap(), p = /* @__PURE__ */ new Map(), m, h, g = (t) => /* @__PURE__ */ Error(`figure "${e}": ${t}`), _ = (e, t) => {
		if (e.length === 0) throw g(`${t} produced an empty id`);
		if (!ir.test(e)) throw g(`id "${e}" (${t}) may only contain letters, digits, "_", ".", ":" and "-"`);
		let r = n.get(e);
		if (r !== void 0) throw g(`duplicate id "${e}" (first created by ${r}, again by ${t})`);
		n.set(e, t);
	}, v = (e) => {
		if (!n.has(e)) return e;
		for (let t = 2;; t += 1) {
			let r = `${e}-${t}`;
			if (!n.has(r)) return r;
		}
	}, y = (e, t, n) => {
		if (n !== void 0) return n;
		let r = t === void 0 ? "" : or(t);
		return v(r.length === 0 ? e : `${e}-${r}`);
	}, b = (e, t) => {
		if (o.has(e.id)) throw g(`node "${e.id}" is already inside another group and cannot be placed again by ${t}; create a second node instead of reusing the object`);
		o.add(e.id);
	}, x = (e, t) => {
		let n = (e, r) => {
			if (!r && i.has(e)) {
				b(e, t);
				return;
			}
			if (_(e.id, t), e.type === "group") for (let t of e.children) n(t, !1);
		};
		return n(e, !0), i.add(e), a.push(e), e;
	}, S = (e, t, i) => {
		if (r.has(e)) {
			if (i) return e;
			throw g(`${t}: "${e}" is an edge, not a node; use f.draw / f.flow / f.pulse for connectors`);
		}
		if (!n.has(e)) throw g(`${t}: unknown target "${e}"`);
		return e;
	}, C = (e, t) => {
		if (r.has(e)) return e;
		throw n.has(e) ? g(`${t}: "${e}" is a node, not an edge`) : g(`${t}: unknown edge "${e}"`);
	}, w = (e, t, r = !1) => {
		if (typeof e == "string") return S(e, t, r);
		if (!lr(e)) return S(e.id, t, r);
		if (!i.has(e) && !n.has(e.id)) throw g(`${t}: node "${e.id}" was not created by this figure; create it with a helper or f.raw() first`);
		return S(e.id, t, r);
	}, T = (e) => {
		for (let t of e) {
			let e = t.id;
			for (let n = 2; d.has(e); n += 1) e = `${t.id}#${n}`;
			d.add(e), u.push(e === t.id ? t : {
				...t,
				id: e
			});
		}
	}, E = (e, t, n = !1) => {
		if (lr(e)) {
			let n = f.get(e);
			if (n !== void 0) return {
				ids: [w(e, t)],
				fragment: n
			};
		} else if (typeof e == "string") {
			let n = p.get(e);
			if (n !== void 0) return {
				ids: [w(e, t)],
				fragment: n
			};
		}
		let r = Array.isArray(e) ? e : [e];
		if (r.length === 0) throw g(`${t}: no targets given`);
		return {
			ids: r.map((e) => w(e, t, n)),
			fragment: void 0
		};
	}, D = (e, t) => {
		let n = Array.isArray(e) ? e : [e];
		if (n.length === 0) throw g(`${t}: no edges given`);
		return n.map((e) => C(typeof e == "string" ? e : e.id, t));
	}, O = (e, t, n, r) => {
		let i = (e) => t.flatMap((t, i) => r(t, e + i * n).map(pr));
		return {
			kind: "motion",
			label: e,
			duration: vn(i(0)),
			tracks: i
		};
	}, k = (e, t) => ({
		kind: "motion",
		label: `reveal(${e.scope})`,
		duration: e.duration,
		tracks: (n) => hr(_n(e.tracks, n), t)
	}), A = (e, t) => {
		if (!Number.isFinite(e) || e < 0) throw g(`${t}: time must be a finite, non-negative number of milliseconds`);
		return e;
	}, j = (e, t, n, r = {}) => {
		let { id: i, textStyle: a, ...o } = r, s = y(e, n, i), c = a ?? t;
		return x(zn(s, n, {
			...o,
			...c === void 0 ? {} : { textStyle: c }
		}), cr(e, n));
	}, M = (e, t, n, r = {}) => {
		let { id: i, ...a } = r, o = y(e, a.label, i);
		return x(qn(o, t, n, a), cr(e, a.label));
	};
	function N(e, t = {}) {
		if (ur(e)) return M("flow", {
			wide: "row",
			compact: "stack"
		}, e, t);
		let { duration: n, stagger: r = 0, easing: i } = t, a = D(e, "f.flow");
		return O(`flow(${a.join(",")})`, a, r, (e, t) => hr([Ie(e, t, n === void 0 ? void 0 : t + n)], i));
	}
	return {
		builder: {
			text: (e, t) => j("text", void 0, e, t),
			eyebrow: (e, t) => j("eyebrow", "label", e, t),
			heading: (e, t) => j("heading", "bodyStrong", e, t),
			title: (e, t) => j("title", "title", e, t),
			caption: (e, t) => j("caption", "caption", e, {
				maxLines: 4,
				...t
			}),
			body: (e, t) => j("body", "body", e, t),
			code: (e, t) => j("code", "code", e, t),
			badge(e, t = {}) {
				let { id: n, ...r } = t, i = y("badge", e, n);
				return x({
					id: i,
					type: "badge",
					text: e,
					...r
				}, cr("badge", e));
			},
			icon(e, t = {}) {
				let { id: n, ...r } = t, i = y("icon", e, n);
				return x({
					id: i,
					type: "icon",
					icon: e,
					...r
				}, cr("icon", e));
			},
			rect(e = {}) {
				let { id: t, ...n } = e, r = y("rect", n.label, t);
				return x({
					id: r,
					type: "rect",
					...n
				}, cr("rect", n.label));
			},
			circle(e = {}) {
				let { id: t, ...n } = e, r = y("circle", n.label, t);
				return x({
					id: r,
					type: "circle",
					...n
				}, cr("circle", n.label));
			},
			polyline(e, t = {}) {
				let { id: n, ...r } = t, i = y("polyline", r.label, n);
				return x({
					id: i,
					type: "polyline",
					points: e,
					...r
				}, cr("polyline", r.label));
			},
			path(e, t, n = {}) {
				let { id: r, ...i } = n, a = y("path", i.label, r);
				return x({
					id: a,
					type: "path",
					d: e,
					viewBox: t,
					...i
				}, cr("path", i.label));
			},
			image(e, t, n = {}) {
				let { id: r, ...i } = n, a = y("image", t, r);
				return x({
					id: a,
					type: "image",
					src: e,
					alt: t,
					...i
				}, cr("image", t));
			},
			legend(e, t = {}) {
				let { id: n, ...r } = t, i = y("legend", r.label, n);
				return x({
					id: i,
					type: "legend",
					items: e,
					...r
				}, cr("legend", r.label));
			},
			callout(e, t = {}) {
				let { id: n, ...r } = t, i = y("callout", e, n);
				return x({
					id: i,
					type: "callout",
					text: e,
					...r
				}, cr("callout", e));
			},
			card(e) {
				let { id: t, ...n } = e, r = y("card", n.title, t);
				return x(Xn(r, n), cr("card", n.title));
			},
			panel(e, t = {}) {
				let { id: n, ...r } = t, i = r.title ?? r.eyebrow ?? r.label, a = y("panel", i, n);
				return x($n(a, e, r), cr("panel", i));
			},
			pill(e, t = {}) {
				let { id: n, ...r } = t, i = y("pill", e, n);
				return x(Gn(i, e, r), cr("pill", e));
			},
			keyValue(e, t, n = {}) {
				let { id: r, ...i } = n, a = y("key-value", e, r);
				return x(nr(a, e, t, i), cr("keyValue", e));
			},
			rule(e = {}) {
				let t = y("rule", void 0, e.id);
				return x(er(t, e.tone), cr("rule"));
			},
			spacer(e, t = {}) {
				let n = y("spacer", void 0, t.id);
				return x(tr(n, e), cr("spacer"));
			},
			stack: (e, t) => M("stack", "stack", e, t),
			row: (e, t) => M("row", "row", e, t),
			grid: (e, t) => M("grid", "grid", e, t),
			overlay: (e, t) => M("overlay", "overlay", e, t),
			coordinates: (e, t) => M("coordinates", "coordinates", e, t),
			absolute: (e, t) => M("absolute", "absolute", e, t),
			flow: N,
			add(e, t = {}) {
				let n = dr(e) ? e : e.fragment, i = (n.diagnostics ?? []).filter((e) => e.severity === "error");
				if (i.length > 0) throw g(`f.add: the fragment reports errors:\n${i.map((e) => `- ${e.message}`).join("\n")}`);
				if (n.nodes.length === 0) throw g("f.add: the fragment has no nodes");
				let a = n.nodes.length === 1 ? n.nodes[0] : void 0, o = a === void 0 ? "fragment" : a.id.split(":")[0] ?? "", u = t.id ?? v(o.length === 0 ? "fragment" : o), d = _r(n, u);
				if (!d && fr(e)) throw g(`f.add(${JSON.stringify(u)}): this compiler result exposes stable handles, so its ids cannot be re-scoped; set the id when compiling it (for example plot(rows, { id: ${JSON.stringify(u)}, ... })) and call f.add(result) without a different id`);
				let m = d ? n : gn(n, u), h = `f.add(${JSON.stringify(u)})`, y = m.nodes.map((e) => x(e, h));
				for (let e of m.edges ?? []) _(e.id, h), r.add(e.id), s.push(e);
				for (let e of m.controls ?? []) {
					if (l.has(e.id)) throw g(`duplicate control id "${e.id}" (added by ${h})`);
					l.add(e.id), c.push(e);
				}
				let b = m.tracks ?? [], S = {
					scope: u,
					tracks: b,
					duration: vn(b)
				};
				t.at !== void 0 && T(_n(b, A(t.at, "f.add")));
				let C;
				if (y.length === 1 && y[0] !== void 0) C = y[0];
				else {
					let e = m.nodes.some((e) => e.id === u) ? v(`${u}-group`) : u;
					C = x(z(e, y, {
						gap: 0,
						width: "fill"
					}), h);
				}
				return f.set(C, S), p.set(C.id, S), C;
			},
			raw(e) {
				return i.has(e) ? e : x(e, `f.raw(${JSON.stringify(e.id)})`);
			},
			connect(e, t, n = {}) {
				let i = "f.connect", a = (e) => {
					if (typeof e == "string" || lr(e)) return w(e, i);
					let { node: t, ...n } = e;
					return {
						node: w(t, i),
						...n
					};
				}, o = a(e), c = a(t), { id: l, ...u } = n, d = bt(o), f = bt(c), p = l ?? v(`${d}-${f}`);
				_(p, `${i}(${JSON.stringify(d)}, ${JSON.stringify(f)})`), r.add(p);
				let m = {
					id: p,
					from: o,
					to: c,
					...u
				};
				return s.push(m), m;
			},
			reveal(e, t = {}) {
				let { duration: n = ar.reveal, stagger: r = 0, offset: i, scale: a, easing: o } = t, s = E(e, "f.reveal", i === void 0 && a === void 0);
				return s.fragment !== void 0 && s.fragment.tracks.length > 0 ? k(s.fragment, o) : O(`reveal(${s.ids.join(",")})`, s.ids, r, (e, t) => hr(Pe(e, t, t + n, {
					...a === void 0 ? {} : { scale: a },
					...i === void 0 ? {} : { offset: i }
				}), o));
			},
			draw(e, t = {}) {
				let { duration: n = ar.draw, stagger: r = 0, easing: i } = t, a = D(e, "f.draw");
				return O(`draw(${a.join(",")})`, a, r, (e, t) => hr(Fe(e, t, t + n), i));
			},
			pulse(e, t = {}) {
				let { duration: n = ar.pulse, stagger: r = 0, easing: i } = t, a = E(e, "f.pulse", !0);
				return O(`pulse(${a.ids.join(",")})`, a.ids, r, (e, t) => hr([Re(e, t, n)], i));
			},
			highlight(e, t = {}) {
				let { duration: n = ar.highlight, stagger: r = 0, peak: i = 1, rest: a = i, easing: o } = t, s = E(e, "f.highlight", !0);
				return O(`highlight(${s.ids.join(",")})`, s.ids, r, (e, t) => hr([Le(e, t, t + n, i, a)], o));
			},
			progress(e, t = {}) {
				let { duration: n = ar.progress, stagger: r = 0, from: i = 0, to: a = 1, easing: o } = t, s = E(e, "f.progress", !0);
				return O(`progress(${s.ids.join(",")})`, s.ids, r, (e, t) => hr([ze(e, t, t + n, i, a)], o));
			},
			rise(e, t = {}) {
				let { duration: n = ar.rise, stagger: r = 0, easing: i } = t, a = E(e, "f.rise");
				return O(`rise(${a.ids.join(",")})`, a.ids, r, (e, t) => [mr(e, "revealY", t, t + n, 0, 1, i)]);
			},
			wipe(e, t = {}) {
				let { duration: n = ar.wipe, stagger: r = 0, easing: i } = t, a = E(e, "f.wipe");
				return O(`wipe(${a.ids.join(",")})`, a.ids, r, (e, t) => [mr(e, "revealX", t, t + n, 0, 1, i)]);
			},
			sequence(e, t = {}) {
				let n = t.gap ?? ar.gap, r = A(t.start ?? 0, "f.sequence");
				for (let t of e) {
					let e = Array.isArray(t) ? t : [t], i = 0;
					for (let t of e) T(t.tracks(r)), i = Math.max(i, t.duration);
					r += i + n;
				}
			},
			at(e, ...t) {
				let n = A(e, "f.at");
				for (let e of t) T(e.tracks(n));
			},
			machine(t) {
				if (m !== void 0) throw g("f.machine was called twice; merge the definitions");
				let { id: n, ...r } = t;
				m = {
					id: n ?? `${e}-machine`,
					...r
				};
			},
			controls(e) {
				for (let t of e) {
					let { id: e, ...n } = t, r = e;
					if (r === void 0) {
						let e = or(t.label) || "control";
						r = e;
						for (let t = 2; l.has(r); t += 1) r = `${e}-${t}`;
					} else if (l.has(r)) throw g(`duplicate control id "${r}"`);
					l.add(r), c.push({
						id: r,
						...n
					});
				}
			},
			root(e) {
				if (h !== void 0) throw g("f.root was called twice");
				if (!i.has(e)) x(e, "f.root(…)");
				else if (o.has(e.id)) throw g(`f.root: "${e.id}" is nested inside another group and cannot be the root`);
				h = e;
			}
		},
		finish: () => {
			let r = h;
			if (r === void 0) {
				let e = a.filter((e) => !o.has(e.id));
				if (e.length === 0) throw g("no nodes were created; add content or call f.root(...)");
				r = x(z(v("root"), e, {
					gap: 16,
					width: "fill"
				}), "root (inferred)");
			}
			let i = /* @__PURE__ */ new Set();
			yt(r, (e) => i.add(e.id));
			let l = a.filter((e) => !o.has(e.id) && !i.has(e.id));
			if (l.length > 0) {
				let e = l.map((e) => `"${e.id}" (${n.get(e.id) ?? "?"})`).join(", "), [t, r] = l.length === 1 ? ["is", "it"] : ["are", "them"];
				throw g(`${e} ${t} not inside the root; add ${r} to a group or pass ${r} to f.root(...)`);
			}
			if (c.length > 0 && m === void 0) throw g("controls need a state machine; call f.machine(...)");
			if (m !== void 0) {
				let e = Tn(m, { nodeIds: i }).diagnostics.filter((e) => e.severity === "error");
				if (e.length > 0) throw g(`invalid machine:\n${e.map((e) => `- ${e.message}`).join("\n")}`);
			}
			let d = m === void 0 ? void 0 : /* @__PURE__ */ new Set([
				...Object.keys(m.variables ?? {}),
				...Object.keys(m.signals ?? {}),
				"$state",
				"$selection"
			]), f = (e, t) => {
				for (let [n, r] of t) {
					if (d === void 0) throw g(`"${e}" binds ${n} to signal "${r}" but the figure has no machine; call f.machine(...)`);
					if (!d.has(r)) throw g(`"${e}" binds ${n} to unknown signal "${r}"; declare it in the machine's signals or variables`);
				}
			};
			yt(r, (e) => f(e.id, vr(e.bind)));
			for (let e of s) {
				f(e.id, vr(e.bind));
				for (let t of e.labels ?? []) f(`${e.id} label`, vr(t.bind));
			}
			let p;
			return u.length > 0 && (p = {
				duration: vn(u) + (t.hold ?? 0),
				tracks: [...u]
			}), Ct({
				schemaVersion: 2,
				id: e,
				title: t.title,
				...t.description === void 0 ? {} : { description: t.description },
				...t.breakpoints === void 0 ? {} : { breakpoints: t.breakpoints },
				...t.padding === void 0 ? {} : { padding: t.padding },
				...t.background === void 0 ? {} : { background: t.background },
				root: r,
				...s.length === 0 ? {} : { edges: [...s] },
				...p === void 0 ? {} : { timeline: p },
				...m === void 0 ? {} : { machine: m },
				...c.length === 0 ? {} : { controls: [...c] },
				...t.metadata === void 0 ? {} : { metadata: t.metadata }
			});
		}
	};
}
function br(e, t, n) {
	if (e.length === 0 || !ir.test(e)) throw Error(`figure id "${e}" may only contain letters, digits, "_", ".", ":" and "-" and must not be empty`);
	if (t.title.length === 0) throw Error(`figure "${e}": title must not be empty`);
	let { builder: r, finish: i } = yr(e, t);
	return n(r), i();
}
//#endregion
//#region ../core/dist/layout.js
var xr = {
	gap: 24,
	stackedGap: 16,
	minItemWidth: 168,
	preferredItemWidth: 224,
	maxItemWidth: 320,
	itemHeight: 128,
	maxStackedWidth: 640,
	padding: 24,
	precision: 3
};
function Sr(e, t) {
	if (!Number.isFinite(e) || e < 0) throw RangeError(`${t} must be a finite, non-negative number`);
	return e;
}
function Cr(e) {
	if (typeof e == "number") {
		let t = Sr(e, "padding");
		return {
			top: t,
			right: t,
			bottom: t,
			left: t
		};
	}
	return {
		top: Sr(e?.top ?? xr.padding, "padding.top"),
		right: Sr(e?.right ?? xr.padding, "padding.right"),
		bottom: Sr(e?.bottom ?? xr.padding, "padding.bottom"),
		left: Sr(e?.left ?? xr.padding, "padding.left")
	};
}
function wr(e, t) {
	let n = 10 ** t;
	return Math.round((e + 2 ** -52) * n) / n;
}
function Tr(e, t) {
	let n = e.map((e) => e.min), r = t - n.reduce((e, t) => e + t, 0), i = (t, i) => {
		for (; r > 1e-9;) {
			let a = e.map((e, r) => ({
				index: r,
				capacity: t(e) - (n[r] ?? 0),
				weight: i ? e.grow : 1
			})).filter(({ capacity: e, weight: t }) => e > 1e-9 && t > 0);
			if (a.length === 0) return;
			let o = a.reduce((e, t) => e + t.weight, 0), s = 0;
			for (let e of a) {
				let t = Math.min(e.capacity, r * (e.weight / o));
				n[e.index] = (n[e.index] ?? 0) + t, s += t;
			}
			if (s <= 1e-9) return;
			r -= s;
		}
	};
	return i((e) => e.preferred, !1), i((e) => e.max, !0), n;
}
function Er(e, t, n = {}) {
	Sr(e, "containerWidth");
	let r = n.precision ?? xr.precision;
	if (!Number.isInteger(r) || r < 0 || r > 10) throw RangeError("precision must be an integer between 0 and 10");
	let i = /* @__PURE__ */ new Set();
	for (let e of t) {
		if (e.id.length === 0) throw Error("pipeline item ids must not be empty");
		if (i.has(e.id)) throw Error(`duplicate pipeline item id: ${e.id}`);
		i.add(e.id);
	}
	let a = Cr(n.padding), o = Sr(n.gap ?? xr.gap, "gap"), s = Sr(n.stackedGap ?? xr.stackedGap, "stackedGap"), c = Sr(n.minItemWidth ?? xr.minItemWidth, "minItemWidth"), l = Sr(n.preferredItemWidth ?? xr.preferredItemWidth, "preferredItemWidth"), u = Sr(n.maxItemWidth ?? xr.maxItemWidth, "maxItemWidth"), d = Sr(n.itemHeight ?? xr.itemHeight, "itemHeight"), f = Sr(n.maxStackedWidth ?? xr.maxStackedWidth, "maxStackedWidth"), p = Math.max(0, e - a.left - a.right), m = t.map((e, t) => {
		let n = Sr(e.minWidth ?? c, `items[${t}].minWidth`), r = Sr(e.preferredWidth ?? l, `items[${t}].preferredWidth`), i = Sr(e.maxWidth ?? u, `items[${t}].maxWidth`);
		if (n > r || r > i) throw RangeError(`items[${t}] widths must satisfy minWidth <= preferredWidth <= maxWidth`);
		return {
			min: n,
			preferred: r,
			max: i,
			grow: Sr(e.grow ?? 1, `items[${t}].grow`),
			height: Sr(e.height ?? d, `items[${t}].height`)
		};
	}), h = m.reduce((e, t) => e + t.min, 0) + o * Math.max(0, t.length - 1), g = n.mode ?? "auto", _ = g === "auto" ? h <= p ? "wide" : "stacked" : g;
	if (_ === "wide" && h > p) throw RangeError(`wide layout requires at least ${h + a.left + a.right}px`);
	if (t.length === 0) return {
		mode: _,
		size: {
			width: wr(e, r),
			height: wr(a.top + a.bottom, r)
		},
		padding: a,
		items: [],
		connectors: []
	};
	let v = [];
	if (_ === "wide") {
		let e = Tr(m, p - o * Math.max(0, t.length - 1)), n = Math.max(...m.map((e) => e.height)), i = a.left;
		t.forEach((t, s) => {
			let c = e[s] ?? 0, l = m[s]?.height ?? d, u = a.top + (n - l) / 2, f = {
				x: wr(i, r),
				y: wr(u, r),
				width: wr(c, r),
				height: wr(l, r)
			};
			v.push({
				id: t.id,
				index: s,
				bounds: f,
				entry: {
					x: f.x,
					y: wr(f.y + f.height / 2, r)
				},
				exit: {
					x: wr(f.x + f.width, r),
					y: wr(f.y + f.height / 2, r)
				}
			}), i += c + o;
		});
	} else {
		let e = Math.min(p, f), n = a.left + (p - e) / 2, i = a.top;
		t.forEach((t, a) => {
			let o = m[a]?.height ?? d, c = {
				x: wr(n, r),
				y: wr(i, r),
				width: wr(e, r),
				height: wr(o, r)
			};
			v.push({
				id: t.id,
				index: a,
				bounds: c,
				entry: {
					x: wr(c.x + c.width / 2, r),
					y: c.y
				},
				exit: {
					x: wr(c.x + c.width / 2, r),
					y: wr(c.y + c.height, r)
				}
			}), i += o + s;
		});
	}
	let y = v.slice(0, -1).map((e, t) => {
		let n = v[t + 1];
		if (n === void 0) throw Error("unreachable: missing next pipeline item");
		return {
			from: e.id,
			to: n.id,
			start: e.exit,
			end: n.entry
		};
	}), b = v[v.length - 1], x = b === void 0 ? a.top : b.bounds.y + b.bounds.height, S = Math.max(...v.map((e) => e.bounds.y + e.bounds.height), a.top);
	return {
		mode: _,
		size: {
			width: wr(e, r),
			height: wr((_ === "wide" ? S : x) + a.bottom, r)
		},
		padding: a,
		items: v,
		connectors: y
	};
}
//#endregion
//#region ../core/dist/material.js
function B(e = "flat", t = {}) {
	return {
		material: e,
		...t
	};
}
function Dr(e = {}) {
	return {
		type: "shadow",
		...e
	};
}
function Or(e = {}) {
	return {
		type: "shadow",
		kind: "inner",
		...e
	};
}
function kr(e) {
	return {
		type: "blur",
		radius: e
	};
}
function Ar(e = {}) {
	return {
		type: "backdrop",
		...e
	};
}
function jr(e = {}) {
	return {
		type: "noise",
		...e
	};
}
function Mr(e, t = {}) {
	return {
		type: "shader",
		name: e,
		...t.uniforms === void 0 ? {} : { uniforms: t.uniforms },
		...t.fallback === void 0 ? {} : { fallback: t.fallback }
	};
}
//#endregion
//#region ../core/dist/pipeline.js
function Nr(e, t) {
	if (e.length === 0) throw Error(`${t} id must not be empty`);
}
function Pr(e) {
	if (Nr(e.id, "pipeline"), e.title.length === 0) throw Error("pipeline title must not be empty");
	let t = /* @__PURE__ */ new Set();
	for (let n of e.nodes) {
		if (Nr(n.id, "node"), t.has(n.id)) throw Error(`duplicate node id: ${n.id}`);
		t.add(n.id);
	}
	let n = /* @__PURE__ */ new Set();
	for (let r of e.edges) {
		if (Nr(r.id, "edge"), n.has(r.id) || t.has(r.id)) throw Error(`duplicate scene id: ${r.id}`);
		if (n.add(r.id), !t.has(r.from)) throw Error(`edge ${r.id} refers to missing source node ${r.from}`);
		if (!t.has(r.to)) throw Error(`edge ${r.id} refers to missing target node ${r.to}`);
	}
	return {
		...e,
		nodes: [...e.nodes],
		edges: [...e.edges]
	};
}
function Fr(e, t) {
	switch (e.tone ?? "neutral") {
		case "accent": return t.colors.accent;
		case "success": return t.colors.success;
		case "warning": return t.colors.warning;
		case "danger": return t.colors.danger;
		case "neutral": return t.colors.border;
	}
}
function Ir(e) {
	return Math.round((e + 2 ** -52) * 1e3) / 1e3;
}
function Lr(e, t) {
	let n = Pr(e), r = t.theme ?? Mt, i = Er(t.width, n.nodes, {
		...t.layout === void 0 ? {} : { mode: t.layout },
		...t.padding === void 0 ? {} : { padding: t.padding },
		...t.gap === void 0 ? {} : { gap: t.gap },
		...t.stackedGap === void 0 ? {} : { stackedGap: t.stackedGap },
		minItemWidth: 168,
		preferredItemWidth: 224,
		maxItemWidth: 320,
		itemHeight: 128
	}), a = new Map(i.items.map((e) => [e.id, e])), o = n.nodes.map((e) => {
		let t = a.get(e.id);
		if (t === void 0) throw Error(`layout omitted node ${e.id}`);
		return {
			id: e.id,
			kind: "rect",
			...t.bounds,
			label: e.label,
			...e.description === void 0 ? {} : { description: e.description },
			appearance: {
				fill: r.colors.surface,
				stroke: Fr(e, r),
				strokeWidth: 1,
				radius: r.radii.lg
			},
			state: {
				opacity: 1,
				translateX: 0,
				translateY: 0,
				scale: 1,
				progress: 1
			},
			interactive: e.interactive ?? !1,
			focusable: e.interactive ?? !1,
			metadata: e.metadata ?? {}
		};
	}), s = new Map(o.map((e) => [e.id, e])), c = n.edges.map((e) => {
		let t = s.get(e.from), n = s.get(e.to);
		if (t === void 0 || n === void 0) throw Error(`edge ${e.id} has unresolved endpoints`);
		let a = i.mode === "wide", o = a ? {
			x: Ir(t.x + t.width),
			y: Ir(t.y + t.height / 2)
		} : {
			x: Ir(t.x + t.width / 2),
			y: Ir(t.y + t.height)
		}, c = a ? {
			x: Ir(n.x),
			y: Ir(n.y + n.height / 2)
		} : {
			x: Ir(n.x + n.width / 2),
			y: Ir(n.y)
		};
		return {
			id: e.id,
			from: e.from,
			to: e.to,
			start: o,
			end: c,
			path: `M ${o.x} ${o.y} L ${c.x} ${c.y}`,
			directed: e.directed ?? !0,
			...e.label === void 0 ? {} : { label: e.label },
			appearance: {
				stroke: r.colors.connector,
				strokeWidth: 2
			},
			state: {
				opacity: 1,
				progress: 1
			}
		};
	});
	return {
		id: n.id,
		width: i.size.width,
		height: i.size.height,
		label: n.title,
		title: n.title,
		...n.description === void 0 ? {} : { description: n.description },
		layout: i.mode,
		theme: Yt(r),
		nodes: o,
		edges: c,
		...n.timeline === void 0 ? {} : { timeline: n.timeline }
	};
}
//#endregion
//#region ../core/dist/resolve.js
var Rr = {
	wide: 900,
	compact: 560
};
function zr(e, t, n = "auto") {
	if (n !== "auto") return n;
	let r = t?.wide ?? Rr.wide, i = t?.compact ?? Rr.compact;
	return e >= r ? "wide" : e >= i ? "compact" : "narrow";
}
function Br(e, t) {
	return e === void 0 ? {
		top: t,
		right: t,
		bottom: t,
		left: t
	} : typeof e == "number" ? {
		top: e,
		right: e,
		bottom: e,
		left: e
	} : e.length === 2 ? {
		top: e[0],
		right: e[1],
		bottom: e[0],
		left: e[1]
	} : {
		top: e[0],
		right: e[1],
		bottom: e[2],
		left: e[3]
	};
}
function Vr(e, t) {
	let n = t.typography[e];
	return {
		family: n.family,
		size: n.size,
		weight: n.weight,
		lineHeight: n.lineHeight,
		...n.letterSpacing === void 0 ? {} : { letterSpacing: n.letterSpacing }
	};
}
function Hr(e) {
	return e != null && e !== !1 && e !== 0 && e !== "";
}
function Ur(e) {
	return typeof e == "number" && Number.isFinite(e) ? e : void 0;
}
function Wr(e, t) {
	return e === void 0 ? t : Math.min(1, Math.max(0, e));
}
var Gr = "top-left";
function Kr(e, t, n, r, i) {
	let a = e.bind ?? {}, o = (e) => e === void 0 ? void 0 : r[e], s = a.hidden === void 0 ? R(e.hidden, t, !1) : Hr(o(a.hidden)), c = Ur(o(a.width)), l = Ur(o(a.height)), u = c ?? vt(e.width, t) ?? (e.type === "rect" || e.type === "polyline" || e.type === "group" && e.layout === "coordinates" ? "fill" : void 0), d = l ?? vt(e.height, t), f = a.text === void 0 ? void 0 : o(a.text), p = a.tone === void 0 ? void 0 : o(a.tone), m = a.description === void 0 ? void 0 : o(a.description), h = e.z ?? i, g = e.type === "group", _ = vt(e.position, t), v = _ === void 0 ? void 0 : {
		x: _.x,
		y: _.y,
		anchor: _.anchor ?? Gr
	}, y, b, x = n.colors.text, S = "start", C = "none", w = Infinity, T, E = 24, D = 12, O = "none", k = "row";
	switch (typeof p == "string" && p.length > 0 && (T = p), e.type) {
		case "text": {
			let r = R(e.textStyle, t, "body");
			b = Vr(r, n), y = typeof f == "string" ? f : f == null ? e.text : String(f);
			let i = r === "label" || r === "caption" ? n.colors.textMuted : n.colors.text;
			x = Lt(e.color, n, "text", i), S = R(e.align, t, "start"), C = e.transform ?? (r === "label" && n.ornament.eyebrow === !0 ? "uppercase" : "none"), w = e.wrap === !1 ? 1 : R(e.maxLines, t, Infinity);
			break;
		}
		case "badge": {
			let r = R(e.textStyle, t, "label");
			b = Vr(r, n), y = typeof f == "string" ? f : e.text, T ??= e.tone ?? "accent", C = r === "label" && n.ornament.eyebrow === !0 ? "uppercase" : "none", w = 1;
			break;
		}
		case "callout":
			b = Vr(R(e.textStyle, t, "caption"), n), y = typeof f == "string" ? f : e.text, T ??= e.tone ?? "accent", x = n.colors.text, O = R(e.pointer, t, "none"), w = R(e.maxLines, t, 4);
			break;
		case "legend":
			b = Vr(R(e.textStyle, t, "caption"), n), x = n.colors.textMuted, k = R(e.direction, t, "row");
			break;
		case "icon":
			E = R(e.size, t, 24), T ??= e.tone ?? "accent";
			break;
		case "circle": D = R(e.radius, t, 12);
	}
	let A = typeof m == "string" ? m : e.description;
	return {
		node: e,
		id: e.id,
		type: e.type,
		hidden: s,
		width: u,
		height: d,
		minWidth: R(e.minWidth, t, 0),
		maxWidth: R(e.maxWidth, t, Infinity),
		minHeight: R(e.minHeight, t, 0),
		grow: R(e.grow, t, 0),
		alignSelf: vt(e.alignSelf, t),
		justifySelf: vt(e.justifySelf, t),
		position: v,
		z: h,
		opacity: a.opacity === void 0 ? e.opacity ?? 1 : Wr(Ur(o(a.opacity)), 1),
		highlight: a.highlight === void 0 ? 0 : Wr(Ur(o(a.highlight)) ?? +!!Hr(o(a.highlight)), 0),
		progress: a.progress === void 0 ? 1 : Wr(Ur(o(a.progress)), 1),
		tone: T,
		text: y,
		description: A,
		font: b,
		textColor: x,
		textAlign: S,
		transform: C,
		maxLines: w,
		layout: g ? R(e.layout, t, "stack") : "stack",
		gap: g ? R(e.gap, t, n.spacing.sm) : 0,
		padding: Br(g ? vt(e.padding, t) : void 0, 0),
		align: g ? vt(e.align, t) : void 0,
		justify: g ? R(e.justify, t, "start") : "start",
		columns: g ? Math.max(1, Math.floor(R(e.columns, t, 2))) : 1,
		children: g ? e.children.map((e) => Kr(e, t, n, r, h)) : [],
		iconSize: E,
		circleRadius: D,
		pointer: O,
		legendDirection: k
	};
}
var qr = 10, Jr = 4, Yr = 8, Xr = 16;
function Zr(e) {
	let t = e.text ?? "";
	return e.transform === "uppercase" ? t.toUpperCase() : t;
}
function Qr(e) {
	let t = e.node;
	return t.type === "callout" ? Br(vt(t.padding, "wide"), 0) : Br(void 0, 0);
}
function $r(e) {
	return e.children.filter((e) => !e.hidden);
}
function ei(e, t) {
	if (typeof e.width == "number") return e.width;
	switch (e.type) {
		case "text": return e.font === void 0 ? 0 : ni(Zr(e), e.font);
		case "badge": return (e.font === void 0 ? 0 : Ot(Zr(e), e.font)) + 20;
		case "callout": {
			let t = ai(e), n = e.pointer === "left" || e.pointer === "right" ? Yr : 0;
			return (e.font === void 0 ? 0 : ni(Zr(e), e.font)) + t.left + t.right + n;
		}
		case "icon": return e.iconSize;
		case "circle": return e.circleRadius * 2;
		case "path": {
			let t = e.node;
			return t.type === "path" ? typeof e.height == "number" ? e.height * t.viewBox.width / t.viewBox.height : t.viewBox.width : 0;
		}
		case "image": return typeof e.height == "number" ? e.height * 1.6 : 160;
		case "rect":
		case "polyline": return Math.max(e.minWidth, 0);
		case "legend": {
			let n = e.node;
			if (n.type !== "legend" || e.font === void 0) return 0;
			let r = n.items.map((t) => 19 + Ot(t.label, e.font ?? ti)), i = R(n.gap, t, Xr);
			return e.legendDirection === "row" ? r.reduce((e, t) => e + t, 0) + i * Math.max(0, r.length - 1) : Math.max(0, ...r);
		}
		case "group": {
			let n = $r(e), r = e.padding.left + e.padding.right;
			switch (e.layout) {
				case "row": return n.reduce((e, n) => e + ei(n, t), 0) + e.gap * Math.max(0, n.length - 1) + r;
				case "grid": return Math.max(0, ...n.map((e) => ei(e, t))) * e.columns + e.gap * (e.columns - 1) + r;
				case "absolute": return Math.max(0, ...n.map((e) => (e.position?.x ?? 0) + ei(e, t))) + r;
				default: return Math.max(0, ...n.map((e) => ei(e, t))) + r;
			}
		}
	}
}
var ti = {
	family: "sans-serif",
	size: 12,
	weight: 400,
	lineHeight: 16
};
function ni(e, t) {
	return Math.max(0, ...e.split(/\n/).map((e) => Ot(e.trim(), t)));
}
function ri(e, t) {
	return Math.max(0, ...e.split(/\s+/).map((e) => Ot(e, t)));
}
function ii(e, t) {
	if (typeof e.width == "number") return e.width;
	switch (e.type) {
		case "text": return Math.max(e.minWidth, e.font === void 0 ? 0 : Math.min(ri(Zr(e), e.font), ei(e, t)));
		case "badge":
		case "icon":
		case "circle":
		case "legend": return Math.max(e.minWidth, e.type === "legend" ? Math.min(ei(e, t), 96) : ei(e, t));
		case "callout": {
			let t = ai(e), n = e.pointer === "left" || e.pointer === "right" ? Yr : 0;
			return Math.max(e.minWidth, (e.font === void 0 ? 0 : ri(Zr(e), e.font)) + t.left + t.right + n);
		}
		case "path":
		case "image": return Math.max(e.minWidth, Math.min(ei(e, t), 48));
		case "rect":
		case "polyline": return e.minWidth;
		case "group": {
			let n = $r(e), r = e.padding.left + e.padding.right;
			switch (e.layout) {
				case "row": return n.reduce((e, n) => e + ii(n, t), 0) + e.gap * Math.max(0, n.length - 1) + r;
				case "grid": return Math.max(0, ...n.map((e) => ii(e, t))) * e.columns + e.gap * (e.columns - 1) + r;
				case "absolute": return Math.max(0, ...n.map((e) => (e.position?.x ?? 0) + ii(e, t))) + r;
				default: return Math.max(e.minWidth, Math.max(0, ...n.map((e) => ii(e, t))) + r);
			}
		}
	}
}
function ai(e) {
	let t = Qr(e), n = e.node;
	return n.type === "callout" && n.padding !== void 0 ? t : {
		top: 8,
		right: 12,
		bottom: 8,
		left: 12
	};
}
function oi(e, t) {
	let n = (e, t) => Math.min(e.max, Math.max(e.min, t)), r = e.map((e) => n(e, e.basis)), i = e.map((e) => e.grow <= 0 && e.shrink <= 0);
	for (let a = 0; a < 64; a += 1) {
		let a = 0;
		e.forEach((e, t) => {
			a += i[t] ? r[t] ?? 0 : e.basis;
		});
		let o = t - a, s = o > 0, c = e.map((e, t) => ({
			item: e,
			index: t
		})).filter(({ item: e, index: t }) => !i[t] && (s ? e.grow > 0 : e.shrink > 0));
		if (c.length === 0 || Math.abs(o) < 1e-6) {
			e.forEach((e, t) => {
				i[t] || (r[t] = n(e, e.basis));
			});
			break;
		}
		let l = (e) => s ? e.grow : e.shrink * Math.max(e.basis, 1e-6), u = c.reduce((e, { item: t }) => e + l(t), 0);
		if (u <= 0) break;
		let d = !1;
		for (let { item: e, index: t } of c) {
			let a = e.basis + o * (l(e) / u), s = n(e, a);
			r[t] = s, Math.abs(s - a) > 1e-9 && (i[t] = !0, d = !0);
		}
		if (e.forEach((e, t) => {
			!i[t] && !c.some((e) => e.index === t) && (r[t] = n(e, e.basis));
		}), !d) break;
	}
	return r.map((t, r) => {
		let i = e[r];
		return i === void 0 ? t : n(i, t);
	});
}
function si(e) {
	if (typeof e != "string" || !e.endsWith("%")) return;
	let t = Number(e.slice(0, -1));
	return Number.isFinite(t) ? Math.max(0, t / 100) : void 0;
}
function ci(e, t) {
	return si(e.height) === void 0 ? void 0 : t;
}
function li(e, t, n, r) {
	if (typeof e.width == "number") return ui(e, e.width);
	let i = si(e.width);
	return i === void 0 ? e.width === "fill" || n ? ui(e, Math.max(0, t)) : ui(e, Math.min(Math.max(0, t), Math.max(ei(e, r.layout), e.minWidth))) : ui(e, Math.max(0, t) * i);
}
function ui(e, t) {
	return Math.min(e.maxWidth, Math.max(e.minWidth, t));
}
function di(e, t) {
	return e.height === "fill" || (e.alignSelf ?? t) === "stretch";
}
function fi(e, t, n, r, i = !1) {
	let a = {
		view: e,
		x: 0,
		y: 0,
		width: t,
		height: 0,
		children: []
	}, o = si(e.height), s = typeof e.height == "number" ? e.height : o !== void 0 && n !== void 0 ? n * o : void 0, c = (e.height === "fill" || i) && n !== void 0 ? n : void 0;
	switch (e.type) {
		case "text": {
			let n = e.font ?? ti, r = s === void 0 ? e.maxLines : Math.max(1, Math.min(e.maxLines, Math.floor(s / n.lineHeight))), i = Zr(e).split(/\n/), o = [], l = !1;
			for (let e of i) {
				let i = r - o.length;
				if (i <= 0) {
					l = !0;
					break;
				}
				let a = jt(e, t, n, { maxLines: i });
				o.push(...a), a.some((e) => e.text.endsWith("…")) && (l = !0);
			}
			a.lines = o, a.truncated = l, a.height = s ?? c ?? Math.max(e.minHeight, o.length * n.lineHeight);
			break;
		}
		case "badge": {
			let n = e.font ?? ti, r = jt(Zr(e), Math.max(1, t - 20), n, { maxLines: 1 });
			a.lines = r, a.truncated = r.some((e) => e.text.endsWith("…")), a.height = s ?? Math.max(e.minHeight, n.lineHeight + 8);
			break;
		}
		case "callout": {
			let n = e.font ?? ti, r = ai(e), i = e.pointer === "left" || e.pointer === "right" ? Yr : 0, o = e.pointer === "up" || e.pointer === "down" ? Yr : 0, c = Math.max(1, t - r.left - r.right - i), l = jt(Zr(e), c, n, { maxLines: e.maxLines });
			a.lines = l, a.truncated = l.some((e) => e.text.endsWith("…"));
			let u = l.length * n.lineHeight + r.top + r.bottom;
			a.height = s ?? Math.max(e.minHeight, u + o);
			let d = {
				x: e.pointer === "left" ? Yr : 0,
				y: e.pointer === "up" ? Yr : 0,
				width: t - i,
				height: a.height - o
			};
			a.calloutBody = d, a.calloutTip = e.pointer === "up" ? {
				x: d.x + Math.min(24, d.width / 2),
				y: 0
			} : e.pointer === "down" ? {
				x: d.x + Math.min(24, d.width / 2),
				y: a.height
			} : e.pointer === "left" ? {
				x: 0,
				y: d.y + Math.min(18, d.height / 2)
			} : e.pointer === "right" ? {
				x: t,
				y: d.y + Math.min(18, d.height / 2)
			} : {
				x: d.x,
				y: d.y
			};
			break;
		}
		case "legend": {
			let n = e.node, i = e.font ?? ti;
			if (n.type !== "legend") break;
			let o = R(n.gap, r.layout, Xr), c = n.items.map((e) => ({
				item: {
					id: e.id,
					label: e.label,
					swatch: e.swatch,
					shape: e.shape ?? "square"
				},
				width: 19 + Ot(e.label, i)
			})), l = [];
			if (e.legendDirection === "column") c.forEach((e, n) => {
				l.push({
					item: e.item,
					box: {
						x: 0,
						y: n * (i.lineHeight + o / 2),
						width: Math.min(t, e.width),
						height: i.lineHeight
					}
				});
			}), a.height = s ?? Math.max(e.minHeight, c.length * i.lineHeight + Math.max(0, c.length - 1) * (o / 2));
			else {
				let n = 0, r = 0;
				for (let e of c) n > 0 && n + e.width > t + 1e-6 && (n = 0, r += 1), l.push({
					item: e.item,
					box: {
						x: n,
						y: r * (i.lineHeight + 4),
						width: Math.min(t, e.width),
						height: i.lineHeight
					}
				}), n += e.width + o;
				a.height = s ?? Math.max(e.minHeight, (r + 1) * i.lineHeight + r * 4);
			}
			a.legendItems = l;
			break;
		}
		case "icon":
			a.height = s ?? c ?? Math.max(e.minHeight, e.iconSize);
			break;
		case "circle":
			a.height = s ?? c ?? Math.max(e.minHeight, typeof e.width == "number" ? e.width : e.circleRadius * 2);
			break;
		case "path": {
			let n = e.node, r = n.type === "path" ? n.viewBox.height / n.viewBox.width : 1;
			a.height = s ?? c ?? Math.max(e.minHeight, t * r);
			break;
		}
		case "image":
			a.height = s ?? c ?? Math.max(e.minHeight, t * .625);
			break;
		case "polyline":
			a.height = s ?? c ?? Math.max(e.minHeight, 48);
			break;
		case "rect":
			a.height = s ?? c ?? Math.max(e.minHeight, 8);
			break;
		case "group": pi(e, a, t, s ?? c, r);
	}
	return a;
}
function pi(e, t, n, r, i) {
	let a = e.padding, o = Math.max(0, n - a.left - a.right), s = r === void 0 ? void 0 : Math.max(0, r - a.top - a.bottom), c = $r(e), l = e.gap, u = [], d = 0, f = (t, n, r) => {
		let i = t.view.alignSelf ?? e.align ?? r, a = e.layout === "row" ? t.height : t.width;
		switch (i) {
			case "center": return (n - a) / 2;
			case "end": return n - a;
			default: return 0;
		}
	}, p = (t, n) => {
		if (n <= 0 || t === 0) return {
			lead: 0,
			between: 0
		};
		switch (e.justify) {
			case "center": return {
				lead: n / 2,
				between: 0
			};
			case "end": return {
				lead: n,
				between: 0
			};
			case "between": return t > 1 ? {
				lead: 0,
				between: n / (t - 1)
			} : {
				lead: n / 2,
				between: 0
			};
			case "around": return {
				lead: n / (t * 2),
				between: n / t
			};
			case "evenly": return {
				lead: n / (t + 1),
				between: n / (t + 1)
			};
			default: return {
				lead: 0,
				between: 0
			};
		}
	};
	switch (e.layout) {
		case "stack": {
			let t = e.align === "stretch", n = c.map((e) => fi(e, li(e, o, t || e.alignSelf === "stretch", i), ci(e, s), i)), r = c.filter((e) => e.height === "fill"), m = l * Math.max(0, c.length - 1), h = n.reduce((e, t) => e + (t.view.height === "fill" ? 0 : t.height), 0), g = n.map((e) => e.height);
			if (s !== void 0 && r.length > 0) {
				let e = Math.max(0, s - h - m), t = r.reduce((e, t) => e + Math.max(t.grow, 1), 0);
				g = n.map((n) => n.view.height === "fill" ? e * Math.max(n.view.grow, 1) / t : n.height);
			}
			let _ = n.map((e, t) => {
				let n = g[t] ?? e.height;
				return Math.abs(n - e.height) > 1e-6 ? fi(e.view, e.width, n, i, !0) : e;
			}), v = _.reduce((e, t) => e + t.height, 0) + m, y = s ?? v, { lead: b, between: x } = p(_.length, y - v), S = a.top + b;
			for (let e of _) e.x = a.left + f(e, o, "start"), e.y = S, S += e.height + l + x, u.push(e);
			d = v;
			break;
		}
		case "row": {
			let n = oi(c.map((e) => {
				let t = si(e.width);
				if (typeof e.width == "number" || t !== void 0) {
					let n = ui(e, typeof e.width == "number" ? e.width : o * (t ?? 0));
					return {
						basis: n,
						min: n,
						max: n,
						grow: 0,
						shrink: 0
					};
				}
				let n = Math.min(ii(e, i.layout), e.maxWidth);
				if (e.width === "fill") return {
					basis: 0,
					min: Math.max(e.minWidth, n),
					max: e.maxWidth,
					grow: e.grow > 0 ? e.grow : 1,
					shrink: 0
				};
				let r = ui(e, Math.max(ei(e, i.layout), e.minWidth));
				return {
					basis: r,
					min: Math.min(r, Math.max(e.minWidth, n)),
					max: e.grow > 0 ? e.maxWidth : r,
					grow: e.grow,
					shrink: 1
				};
			}), o - l * Math.max(0, c.length - 1)), r = c.map((e, t) => fi(e, n[t] ?? 0, ci(e, s), i)), m = s ?? Math.max(0, ...r.map((e) => e.height)), h = r.map((t) => di(t.view, e.align) && typeof t.view.height != "number" && Math.abs(t.height - m) > 1e-6 ? fi(t.view, t.width, m, i, !0) : t), g = h.reduce((e, t) => e + t.width, 0) + l * Math.max(0, h.length - 1), { lead: _, between: v } = p(h.length, o - g), y = a.left + _;
			for (let e of h) e.x = y, e.y = a.top + f(e, m, "start"), y += e.width + l + v, u.push(e);
			g > o + .5 && (t.overflowX = !0, i.diagnostics.push({
				severity: "warning",
				code: "overflow",
				message: `row ${e.id} content (${V(g, 1)}px) exceeds its ${V(o, 1)}px inner width in the ${i.layout} layout`,
				path: e.id
			})), d = m;
			break;
		}
		case "grid": {
			let t = e.columns, n = Math.max(0, (o - l * (t - 1)) / t), r = c.map((t) => fi(t, li(t, n, e.align === "stretch" || t.alignSelf === "stretch" || (t.justifySelf ?? void 0) === "stretch", i), ci(t, s), i)), p = a.top;
			for (let o = 0; o < r.length; o += t) {
				let s = r.slice(o, o + t), c = Math.max(0, ...s.map((e) => e.height));
				s.forEach((t, r) => {
					let o = di(t.view, e.align) && typeof t.view.height != "number" && Math.abs(t.height - c) > 1e-6 ? fi(t.view, t.width, c, i, !0) : t, s = a.left + r * (n + l), d = o.view.justifySelf ?? mi(e.justify);
					o.x = s + (d === "center" ? (n - o.width) / 2 : d === "end" ? n - o.width : 0), o.y = p + f(o, c, "start"), u.push(o);
				}), p += c + l;
			}
			d = Math.max(0, p - a.top - l);
			break;
		}
		case "overlay": {
			let t = c.map((e) => fi(e, li(e, o, e.alignSelf === "stretch" || (e.justifySelf ?? void 0) === "stretch", i), ci(e, s), i)), n = s ?? Math.max(0, ...t.map((e) => e.height));
			for (let r of t) {
				let t = (r.view.height === "fill" || r.view.alignSelf === "stretch") && typeof r.view.height != "number" && Math.abs(r.height - n) > 1e-6 ? fi(r.view, r.width, n, i, !0) : r, s = t.view.justifySelf ?? mi(e.justify, "center"), c = t.view.alignSelf ?? e.align ?? "center";
				t.x = a.left + (s === "center" ? (o - t.width) / 2 : s === "end" ? o - t.width : 0), t.y = a.top + (c === "center" ? (n - t.height) / 2 : c === "end" ? n - t.height : 0), u.push(t);
			}
			d = n;
			break;
		}
		case "coordinates": {
			let t = s;
			t === void 0 && (t = Math.max(0, e.minHeight - a.top - a.bottom) || 160, i.diagnostics.push({
				severity: "warning",
				code: "coordinates-height",
				message: `coordinates group ${e.id} has no height; using ${V(t, 1)}px`,
				path: e.id
			}));
			for (let e of c) {
				let n = e.position ?? {
					x: 0,
					y: 0,
					anchor: "top-left"
				}, r = si(e.width), s = typeof e.width == "number" ? e.width : r === void 0 ? e.width === "fill" ? o : li(e, o, !1, i) : o * r, c = si(e.height), l = c === void 0 ? e.height === "fill" ? t : void 0 : t * c, d = fi(c === void 0 ? e : {
					...e,
					height: "fill"
				}, s, l, i), f = hi(n.anchor, d.width, d.height);
				d.x = a.left + n.x * o - f.x, d.y = a.top + n.y * t - f.y, u.push(d);
			}
			d = t;
			break;
		}
		case "absolute": {
			let e = 0;
			for (let t of c) {
				let n = t.position ?? {
					x: 0,
					y: 0,
					anchor: "top-left"
				}, r = fi(t, li(t, Math.max(0, o - n.x), !1, i), s === void 0 ? void 0 : Math.max(0, s - n.y), i), c = hi(n.anchor, r.width, r.height);
				r.x = a.left + n.x - c.x, r.y = a.top + n.y - c.y, e = Math.max(e, r.y + r.height - a.top), u.push(r);
			}
			d = e;
			break;
		}
	}
	t.children.push(...u);
	for (let n of e.children) {
		if (!n.hidden) continue;
		let e = {
			...i,
			diagnostics: []
		}, r = fi(n, li(n, o, !1, e), void 0, e);
		r.x = a.left, r.y = a.top, t.children.push(r);
	}
	t.height = r ?? Math.max(e.minHeight, d + a.top + a.bottom);
	let m = t.height - a.bottom + .5, h = n - a.right + .5, g = e.node.type === "group" && e.node.allowOverflow === !0;
	for (let n of u) {
		if (g) break;
		if (n.x + n.width > h || n.y + n.height > m || n.x < a.left - .5 || n.y < a.top - .5) {
			if (e.layout === "row" && t.overflowX === !0) continue;
			i.diagnostics.push({
				severity: "warning",
				code: "overflow",
				message: `${n.view.type} ${n.view.id} extends outside the content box of ${e.id} in the ${i.layout} layout`,
				path: n.view.id
			});
		}
	}
	if (e.layout !== "overlay" && e.layout !== "coordinates") for (let t = 0; t < u.length; t += 1) for (let n = t + 1; n < u.length; n += 1) {
		let r = u[t], a = u[n];
		r !== void 0 && a !== void 0 && ut(r, a, .5) && i.diagnostics.push({
			severity: "warning",
			code: "overlap",
			message: `${r.view.id} overlaps ${a.view.id} inside ${e.id} in the ${i.layout} layout`,
			path: e.id
		});
	}
}
function mi(e, t = "start") {
	switch (e) {
		case "center": return "center";
		case "end": return "end";
		case "start": return "start";
		default: return t;
	}
}
function hi(e, t, n) {
	switch (e) {
		case "top": return {
			x: t / 2,
			y: 0
		};
		case "top-right": return {
			x: t,
			y: 0
		};
		case "left": return {
			x: 0,
			y: n / 2
		};
		case "center": return {
			x: t / 2,
			y: n / 2
		};
		case "right": return {
			x: t,
			y: n / 2
		};
		case "bottom-left": return {
			x: 0,
			y: n
		};
		case "bottom": return {
			x: t / 2,
			y: n
		};
		case "bottom-right": return {
			x: t,
			y: n
		};
		default: return {
			x: 0,
			y: 0
		};
	}
}
function V(e, t) {
	let n = 10 ** t;
	return Math.round((e + 2 ** -52) * n) / n;
}
function gi(e, t) {
	let n = e.node, r = (e, n) => Lt(e, t, "stroke", n), i = (e, n) => zt(e, t, n), a = (e, t) => ({
		...e,
		...t.effects === void 0 ? {} : { effects: t.effects },
		...t.blendMode === void 0 ? {} : { blendMode: t.blendMode }
	});
	switch (n.type) {
		case "group": {
			let e = n.frame;
			if (e === void 0) return {
				fill: "none",
				stroke: "none",
				strokeWidth: 0,
				radius: 0
			};
			let r = Gt(e, t);
			return a({
				fill: r.fill ?? "none",
				stroke: r.stroke ?? "none",
				strokeWidth: r.strokeWidth ?? t.strokes.hairline,
				radius: r.radius ?? t.radii.lg,
				...r.opacity === void 0 ? {} : { opacity: r.opacity },
				...e.dash === void 0 ? {} : { dash: e.dash }
			}, r);
		}
		case "rect": {
			let o = Gt(n.material, t);
			return a({
				fill: e.tone !== void 0 || n.fill !== void 0 ? i(e.tone ?? n.fill, t.colors.surface) : o.fill ?? t.colors.surface,
				stroke: n.stroke === void 0 ? o.stroke ?? (e.tone === void 0 ? t.colors.border : "none") : r(n.stroke, "none"),
				strokeWidth: n.strokeWidth ?? o.strokeWidth ?? t.strokes.hairline,
				radius: n.radius ?? o.radius ?? t.radii.md,
				...o.opacity === void 0 ? {} : { opacity: o.opacity },
				...n.dash === void 0 ? {} : { dash: n.dash }
			}, o);
		}
		case "circle": {
			let o = Gt(n.material, t);
			return a({
				fill: e.tone !== void 0 || n.fill !== void 0 ? i(e.tone ?? n.fill, t.colors.surface) : o.fill ?? t.colors.surface,
				stroke: n.stroke === void 0 ? o.stroke ?? (e.tone === void 0 ? t.colors.border : "none") : r(n.stroke, "none"),
				strokeWidth: n.strokeWidth ?? o.strokeWidth ?? t.strokes.hairline,
				radius: o.radius ?? 0,
				...o.opacity === void 0 ? {} : { opacity: o.opacity },
				...n.dash === void 0 ? {} : { dash: n.dash }
			}, o);
		}
		case "path": {
			let o = Gt(n.material, t);
			return a({
				fill: e.tone !== void 0 || n.fill !== void 0 ? i(e.tone ?? n.fill, "none") : o.fill ?? "none",
				stroke: n.stroke === void 0 ? o.stroke ?? (e.tone === void 0 && n.fill === void 0 ? t.colors.accent : "none") : r(n.stroke, "none"),
				strokeWidth: n.strokeWidth ?? o.strokeWidth ?? t.strokes.thin,
				radius: o.radius ?? 0,
				...o.opacity === void 0 ? {} : { opacity: o.opacity },
				...n.dash === void 0 ? {} : { dash: n.dash }
			}, o);
		}
		case "polyline": {
			let o = Gt(n.material, t);
			return a({
				fill: n.fill === void 0 ? o.fill ?? "none" : i(n.fill, "none"),
				stroke: e.tone !== void 0 || n.stroke !== void 0 ? r(e.tone ?? n.stroke, "none") : o.stroke ?? (n.fill === void 0 && e.tone === void 0 ? t.colors.accent : "none"),
				strokeWidth: n.strokeWidth ?? o.strokeWidth ?? t.strokes.regular,
				radius: o.radius ?? 0,
				...o.opacity === void 0 ? {} : { opacity: o.opacity },
				...n.dash === void 0 ? {} : { dash: n.dash },
				...n.lineCap === void 0 ? {} : { lineCap: n.lineCap }
			}, o);
		}
		case "image": return {
			fill: "none",
			stroke: "none",
			strokeWidth: 0,
			radius: n.radius ?? t.radii.sm
		};
		case "badge": {
			let r = Lt(e.tone ?? "accent", t, "stroke", t.colors.accent), i = n.variant ?? "soft";
			return i === "solid" ? {
				fill: r,
				stroke: "none",
				strokeWidth: 0,
				radius: t.radii.pill
			} : i === "outline" ? {
				fill: "none",
				stroke: r,
				strokeWidth: t.strokes.hairline,
				radius: t.radii.pill
			} : {
				fill: qt(r, .16),
				stroke: "none",
				strokeWidth: 0,
				radius: t.radii.pill
			};
		}
		case "callout": {
			let n = Lt(e.tone ?? "accent", t, "stroke", t.colors.accent);
			return {
				fill: t.colors.surfaceRaised,
				stroke: n,
				strokeWidth: t.strokes.hairline,
				radius: t.radii.md
			};
		}
		case "icon": {
			let r = Lt(e.tone ?? "accent", t, "stroke", t.colors.accent);
			return {
				fill: Lt(n.background, t, "fill", "none"),
				stroke: r,
				strokeWidth: t.strokes.thin,
				radius: 0
			};
		}
		case "text":
		case "legend": return {
			fill: "none",
			stroke: "none",
			strokeWidth: 0,
			radius: 0
		};
	}
}
function _i(e, t, n, r, i, a) {
	if (t.lines === void 0 || e.font === void 0) return;
	let o = e.font;
	return {
		lines: t.lines.map((e) => ({
			text: e.text,
			width: V(e.width, i)
		})),
		fontFamily: o.family,
		fontSize: o.size,
		fontWeight: o.weight,
		lineHeight: o.lineHeight,
		letterSpacing: o.letterSpacing ?? 0,
		color: a ?? e.textColor,
		align: e.textAlign,
		transform: e.transform,
		box: {
			x: V(n.x + r.x, i),
			y: V(n.y + r.y, i),
			width: V(r.width, i),
			height: V(r.height, i)
		}
	};
}
function vi(e, t, n, r, i, a) {
	let o = e.view, s = t.x + e.x, c = t.y + e.y, l = {
		x: V(s, i),
		y: V(c, i),
		width: V(e.width, i),
		height: V(e.height, i)
	}, u = o.node, d = u.label ?? u.inspect?.title ?? (o.type === "text" || o.type === "badge" || o.type === "callout" ? o.text ?? "" : ""), f = o.description ?? u.inspect?.summary, p = o.type === "group" ? "group" : o.type === "rect" ? "rect" : o.type === "circle" ? "circle" : o.type === "polyline" ? "path" : o.type, m = gi(o, r), h = o.type === "badge" ? ((u.type === "badge" ? u.variant : void 0) ?? "soft") === "solid" ? r.colors.accentContrast : Lt(o.tone ?? "accent", r, "text", r.colors.accent) : void 0, g;
	if (o.type === "text") g = _i(o, e, {
		x: s,
		y: c
	}, {
		x: 0,
		y: 0,
		width: e.width,
		height: e.height
	}, i);
	else if (o.type === "badge") g = _i(o, e, {
		x: s,
		y: c
	}, {
		x: qr,
		y: Jr,
		width: Math.max(0, e.width - 20),
		height: Math.max(0, e.height - 8)
	}, i, h);
	else if (o.type === "callout" && e.calloutBody !== void 0) {
		let t = ai(o);
		g = _i(o, e, {
			x: s,
			y: c
		}, {
			x: e.calloutBody.x + t.left,
			y: e.calloutBody.y + t.top,
			width: Math.max(0, e.calloutBody.width - t.left - t.right),
			height: Math.max(0, e.calloutBody.height - t.top - t.bottom)
		}, i);
	}
	let _ = o.type === "legend" && e.legendItems !== void 0 && o.font !== void 0 ? {
		items: e.legendItems.map((e) => ({
			id: e.item.id,
			label: e.item.label,
			swatch: Lt(e.item.swatch, r, "fill", r.colors.accent),
			shape: e.item.shape,
			box: {
				x: V(s + e.box.x, i),
				y: V(c + e.box.y, i),
				width: V(e.box.width, i),
				height: V(e.box.height, i)
			}
		})),
		text: {
			fontFamily: o.font.family,
			fontSize: o.font.size,
			fontWeight: o.font.weight,
			lineHeight: o.font.lineHeight,
			letterSpacing: o.font.letterSpacing ?? 0,
			color: o.textColor,
			align: "start",
			transform: "none"
		}
	} : void 0, v = o.type === "icon" ? Lt(o.tone ?? "accent", r, "stroke", r.colors.accent) : void 0, y = {
		id: o.id,
		kind: p,
		...l,
		label: d,
		...f === void 0 ? {} : { description: f },
		appearance: m,
		state: {
			opacity: o.opacity,
			translateX: 0,
			translateY: 0,
			scale: 1,
			progress: o.progress,
			highlight: o.highlight
		},
		interactive: u.interactive ?? !1,
		focusable: u.interactive ?? !1,
		metadata: u.metadata ?? {},
		...n === void 0 ? {} : { parent: n },
		z: o.z,
		...o.hidden ? { hidden: !0 } : {},
		...u.type === "group" && u.clip === !0 ? { clip: !0 } : {},
		...u.onActivate === void 0 ? {} : { onActivate: u.onActivate },
		...u.inspect === void 0 ? {} : { inspect: u.inspect },
		...u.focusGroup === !0 ? { focusGroup: !0 } : {},
		...u.revealAnchor === void 0 ? {} : { revealAnchor: u.revealAnchor },
		...g === void 0 ? {} : { text: g },
		...o.type === "icon" && u.type === "icon" ? { icon: {
			name: u.icon,
			size: o.iconSize,
			color: v ?? r.colors.accent,
			background: Lt(u.background, r, "fill", "none")
		} } : {},
		...u.type === "path" ? { path: {
			d: u.d,
			viewBox: u.viewBox
		} } : {},
		...u.type === "polyline" ? { path: {
			d: Ci(u, e.width, e.height, i),
			viewBox: {
				width: Math.max(1e-6, e.width),
				height: Math.max(1e-6, e.height)
			},
			length: V(Ei(u, e.width, e.height), i)
		} } : {},
		...u.type === "image" ? { image: {
			href: u.src,
			alt: u.alt,
			fit: u.fit ?? "contain",
			live: u.live ?? !1
		} } : {},
		..._ === void 0 ? {} : { legend: _ },
		...o.type === "callout" && e.calloutBody !== void 0 && e.calloutTip !== void 0 ? { callout: {
			pointer: o.pointer,
			tip: {
				x: V(s + e.calloutTip.x, i),
				y: V(c + e.calloutTip.y, i)
			},
			body: {
				x: V(s + e.calloutBody.x, i),
				y: V(c + e.calloutBody.y, i),
				width: V(e.calloutBody.width, i),
				height: V(e.calloutBody.height, i)
			}
		} } : {}
	};
	a.nodes.push(y), a.boxes.set(o.id, {
		id: o.id,
		...l,
		kind: p === "circle" ? "circle" : p === "group" ? "group" : p === "rect" ? "rect" : "other"
	}), !o.hidden && o.type !== "group" && a.obstacles.push(l);
	let b = [...e.children].sort((e, t) => e.view.z - t.view.z);
	for (let e of b) vi(e, {
		x: s,
		y: c
	}, o.id, r, i, a);
}
function yi(e, t) {
	let n = Ct(e), r = t.theme ?? Mt, i = t.precision ?? 3;
	if (!Number.isFinite(t.width) || t.width <= 0) throw RangeError("resolveScene width must be a positive, finite number");
	let a = zr(t.width, n.breakpoints, t.layout ?? "auto"), o = [], s, c = {};
	if (n.machine !== void 0) {
		let e = /* @__PURE__ */ new Set(), r = (t) => {
			e.add(t.id), t.type === "group" && t.children.forEach(r);
		};
		r(n.root);
		let i = Tn(n.machine, { nodeIds: e }).diagnostics.filter((e) => e.severity === "error");
		if (i.length > 0) throw Error(`invalid state machine ${n.machine.id}:\n${i.map((e) => `- ${e.message}`).join("\n")}`);
		s = t.machineState ?? Nn(n.machine), c = { ...jn(n.machine, s) };
	}
	t.signals !== void 0 && (c = {
		...c,
		...t.signals
	}), bi(n, c, o);
	let l = Br(vt(n.padding, a), a === "narrow" ? 16 : 24), u = Kr(n.root, a, r, c, 0), d = Math.max(0, t.width - l.left - l.right), f = {
		layout: a,
		theme: r,
		diagnostics: o
	}, p = fi({
		...u,
		width: u.width ?? "fill"
	}, d, typeof u.height == "number" ? u.height : void 0, f);
	p.x = l.left, p.y = l.top;
	let m = {
		nodes: [],
		boxes: /* @__PURE__ */ new Map(),
		obstacles: []
	};
	vi(p, {
		x: 0,
		y: 0
	}, void 0, r, i, m);
	for (let e of m.nodes) e.text !== void 0 && e.text.lines.some((e) => e.text.endsWith("…")) && o.push({
		severity: "warning",
		code: "text-truncated",
		message: `text in ${e.id} was truncated in the ${a} layout`,
		path: e.id
	});
	let h = V(p.height + l.top + l.bottom, i), g = V(t.width, i), _ = n.edges ?? [], v = Qt(_, a, m.boxes), y = Vr("caption", r), b = [];
	for (let e of _) {
		let t = v.get(e.id);
		if (t === void 0) continue;
		let n = e.bind ?? {}, s = n.tone === void 0 ? void 0 : c[n.tone], l = n.hidden === void 0 ? void 0 : Hr(c[n.hidden]), u = n.label === void 0 ? void 0 : c[n.label], d = /* @__PURE__ */ new Set(), f = /* @__PURE__ */ new Map();
		(e.labels ?? []).forEach((t, n) => {
			let r = t.id ?? `${e.id}-label-${n + 1}`;
			if (t.bind?.hidden !== void 0 && Hr(c[t.bind.hidden]) && d.add(r), t.bind?.text !== void 0) {
				let e = c[t.bind.text];
				typeof e == "string" && f.set(r, e);
			}
		}), typeof u == "string" && f.set(`${e.id}-label`, u);
		let p = cn(e, t, {
			layout: a,
			theme: r,
			boxes: m.boxes,
			obstacles: m.obstacles,
			bounds: {
				x: 0,
				y: 0,
				width: g,
				height: h
			},
			labelFont: y,
			labelColor: r.colors.textMuted,
			precision: i,
			overrides: {
				...typeof s == "string" ? { tone: s } : {},
				...l === void 0 ? {} : { hidden: l },
				...typeof u == "string" ? { label: u } : {},
				labelHidden: d,
				labelText: f
			}
		});
		if (p === void 0) continue;
		for (let t of p.collidingLabels) o.push({
			severity: "warning",
			code: "label-collision",
			message: `edge label ${t} overlaps a node in the ${a} layout; hide it per layout, shorten it, or widen the gap`,
			path: e.id
		});
		let _ = n.highlight === void 0 ? void 0 : c[n.highlight], x = n.opacity === void 0 ? void 0 : Ur(c[n.opacity]), S = n.progress === void 0 ? void 0 : Ur(c[n.progress]), C = n.flow === void 0 ? void 0 : c[n.flow], w = _ === void 0 ? 0 : Ur(_) ?? +!!Hr(_), T = C === void 0 ? p.edge.state.flow : Ur(C) ?? +!!Hr(C), E = dn(p.edge.samples ?? [], p.packetCount, p.packetPeriod, 0, i);
		b.push({
			...p.edge,
			state: {
				opacity: Wr(x, 1),
				progress: Wr(S, 1),
				highlight: Wr(w, 0),
				flow: Wr(T, 0)
			},
			packets: E,
			metadata: {
				...p.edge.metadata ?? {},
				packetCount: p.packetCount,
				packetPeriod: p.packetPeriod
			}
		});
	}
	let x = m.nodes.filter((e) => e.hidden !== !0);
	for (let e of x) if (![
		e.x,
		e.y,
		e.width,
		e.height
	].every(Number.isFinite)) throw Error(`node ${e.id} resolved to non-finite geometry`);
	let S = n.background === "transparent" ? "transparent" : Lt(n.background, r, "fill", r.colors.canvas);
	return {
		id: n.id,
		width: g,
		height: h,
		label: n.title,
		title: n.title,
		...n.description === void 0 ? {} : { description: n.description },
		layout: a === "wide" ? "wide" : "stacked",
		layoutName: a,
		theme: Yt(r),
		background: S,
		nodes: m.nodes,
		edges: b,
		...n.timeline === void 0 ? {} : { timeline: n.timeline },
		diagnostics: o,
		...n.machine === void 0 ? {} : { machine: n.machine },
		...s === void 0 ? {} : { machineState: s },
		signals: c,
		...n.controls === void 0 ? {} : { controls: n.controls },
		root: n.root.id
	};
}
function bi(e, t, n) {
	let r = new Set(Object.keys(t)), i = (e, t) => {
		if (t !== void 0) for (let [i, a] of Object.entries(t)) typeof a == "string" && !r.has(a) && n.push({
			severity: "error",
			code: "unknown-signal",
			message: `${e} binds ${i} to unknown signal "${a}"`,
			path: e
		});
	}, a = (e) => {
		i(e.id, e.bind), e.type === "group" && e.children.forEach(a);
	};
	a(e.root);
	for (let t of e.edges ?? []) {
		i(t.id, t.bind);
		for (let e of t.labels ?? []) i(`${t.id} label`, e.bind);
	}
	let o = n.filter((e) => e.code === "unknown-signal");
	if (o.length > 0) throw Error(`invalid bindings in scene ${e.id}:\n${o.map((e) => `- ${e.message}`).join("\n")}`);
}
function xi(e) {
	return e.schemaVersion === 2;
}
function Si(e, t) {
	if (xi(e)) return yi(e, {
		width: t.width,
		layout: t.layout === "stacked" ? "compact" : t.layout ?? "auto",
		...t.theme === void 0 ? {} : { theme: t.theme },
		...t.machineState === void 0 ? {} : { machineState: t.machineState },
		...t.signals === void 0 ? {} : { signals: t.signals },
		...t.precision === void 0 ? {} : { precision: t.precision }
	});
	let n = t.layout ?? "auto", r = n === "wide" || n === "auto" && t.width >= 820 ? "wide" : "stacked", i = Lr(e, {
		width: t.width,
		layout: r,
		...t.theme === void 0 ? {} : { theme: t.theme },
		padding: t.width < 520 ? 16 : 24,
		gap: 24,
		stackedGap: 20
	}), a = i.layout === "wide" ? "wide" : t.width < 560 ? "narrow" : "compact";
	return {
		...i,
		layoutName: a,
		background: i.theme.background
	};
}
function Ci(e, t, n, r = 3) {
	let i = e.space === "px" ? 1 : void 0, a = e.points.map(([e, r]) => ({
		x: i === void 0 ? e * t : e,
		y: i === void 0 ? r * n : r
	}));
	if (a.length === 0) return "";
	let o = (e) => {
		let t = Number(e.toFixed(r));
		return Object.is(t, -0) ? "0" : String(t);
	}, s = a[0];
	if (s === void 0) return "";
	let c = [`M ${o(s.x)} ${o(s.y)}`], l = e.curve ?? "linear";
	if (l === "step") for (let e = 1; e < a.length; e += 1) {
		let t = a[e - 1], n = a[e];
		t !== void 0 && n !== void 0 && c.push(`L ${o(n.x)} ${o(t.y)}`, `L ${o(n.x)} ${o(n.y)}`);
	}
	else if (l === "monotone" && a.length > 2 && wi(a)) {
		let e = Ti(a);
		for (let t = 0; t < a.length - 1; t += 1) {
			let n = a[t], r = a[t + 1];
			if (n === void 0 || r === void 0) continue;
			let i = (r.x - n.x) / 3, s = {
				x: n.x + i,
				y: n.y + i * (e[t] ?? 0)
			}, l = {
				x: r.x - i,
				y: r.y - i * (e[t + 1] ?? 0)
			};
			c.push(`C ${o(s.x)} ${o(s.y)} ${o(l.x)} ${o(l.y)} ${o(r.x)} ${o(r.y)}`);
		}
	} else for (let e = 1; e < a.length; e += 1) {
		let t = a[e];
		t !== void 0 && c.push(`L ${o(t.x)} ${o(t.y)}`);
	}
	if (e.baseline !== void 0) {
		let t = i === void 0 ? e.baseline * n : e.baseline, r = a[a.length - 1];
		r !== void 0 && c.push(`L ${o(r.x)} ${o(t)}`, `L ${o(s.x)} ${o(t)}`, "Z");
	} else e.closed === !0 && c.push("Z");
	return c.join(" ");
}
function wi(e) {
	for (let t = 1; t < e.length; t += 1) {
		let n = e[t - 1], r = e[t];
		if (n === void 0 || r === void 0 || !(r.x > n.x)) return !1;
	}
	return !0;
}
function Ti(e) {
	let t = e.length;
	if (t < 2) return e.map(() => 0);
	let n = [], r = [];
	for (let i = 0; i < t - 1; i += 1) {
		let t = e[i], a = e[i + 1];
		t !== void 0 && a !== void 0 && (n.push(a.x - t.x), r.push((a.y - t.y) / (a.x - t.x)));
	}
	let i = Array(t).fill(0);
	for (let e = 1; e < t - 1; e += 1) {
		let t = r[e - 1] ?? 0, a = r[e] ?? 0, o = n[e - 1] ?? 0, s = n[e] ?? 0;
		if (t * a <= 0) i[e] = 0;
		else {
			let n = 2 * s + o, r = s + 2 * o;
			i[e] = (n + r) / (n / t + r / a);
		}
	}
	let a = (e, t, n, r) => {
		let i = ((2 * n + r) * e - n * t) / (n + r);
		return Math.sign(i) === Math.sign(e) ? Math.sign(e) !== Math.sign(t) && Math.abs(i) > Math.abs(3 * e) && (i = 3 * e) : i = 0, i;
	};
	i[0] = t > 2 ? a(r[0] ?? 0, r[1] ?? 0, n[0] ?? 0, n[1] ?? 0) : r[0] ?? 0, i[t - 1] = t > 2 ? a(r[t - 2] ?? 0, r[t - 3] ?? 0, n[t - 2] ?? 0, n[t - 3] ?? 0) : r[0] ?? 0;
	for (let e = 0; e < t - 1; e += 1) {
		let t = r[e] ?? 0;
		if (t === 0) {
			i[e] = 0, i[e + 1] = 0;
			continue;
		}
		let n = (i[e] ?? 0) / t, a = (i[e + 1] ?? 0) / t, o = n * n + a * a;
		if (o > 9) {
			let r = 3 / Math.sqrt(o);
			i[e] = r * n * t, i[e + 1] = r * a * t;
		}
	}
	return i;
}
function Ei(e, t, n) {
	let r = e.space === "px", i = e.points.map(([e, i]) => ({
		x: r ? e : e * t,
		y: r ? i : i * n
	}));
	if (i.length < 2) return 0;
	let a = 0, o = e.curve ?? "linear", s = o === "monotone" && i.length > 2 && wi(i) ? Ti(i) : void 0;
	for (let e = 1; e < i.length; e += 1) {
		let t = i[e - 1], n = i[e];
		if (t !== void 0 && n !== void 0) {
			if (o === "step") a += Math.abs(n.x - t.x) + Math.abs(n.y - t.y);
			else if (s !== void 0) {
				let r = (n.x - t.x) / 3, i = {
					x: t.x + r,
					y: t.y + r * (s[e - 1] ?? 0)
				}, o = {
					x: n.x - r,
					y: n.y - r * (s[e] ?? 0)
				}, c = t;
				for (let e = 1; e <= 16; e += 1) {
					let r = e / 16, s = 1 - r, l = {
						x: s * s * s * t.x + 3 * s * s * r * i.x + 3 * s * r * r * o.x + r * r * r * n.x,
						y: s * s * s * t.y + 3 * s * s * r * i.y + 3 * s * r * r * o.y + r * r * r * n.y
					};
					a += Math.hypot(l.x - c.x, l.y - c.y), c = l;
				}
			} else a += Math.hypot(n.x - t.x, n.y - t.y);
		}
	}
	let c = i[0], l = i[i.length - 1];
	if (e.baseline !== void 0 && c !== void 0 && l !== void 0) {
		let t = r ? e.baseline : e.baseline * n;
		a += Math.abs(l.y - t) + Math.abs(l.x - c.x) + Math.abs(t - c.y);
	} else e.closed === !0 && c !== void 0 && l !== void 0 && (a += Math.hypot(l.x - c.x, l.y - c.y));
	return a;
}
//#endregion
//#region ../core/dist/seek.js
function Di(e, t, n) {
	return Math.min(n, Math.max(t, e));
}
var Oi = /* @__PURE__ */ new Set([
	"opacity",
	"progress",
	"edgeReveal",
	"highlight",
	"flow"
]), ki = /* @__PURE__ */ new Set([
	"opacity",
	"translateX",
	"translateY",
	"scale",
	"progress",
	"highlight",
	"revealX",
	"revealY"
]);
function Ai(e, t) {
	if (!Number.isFinite(e.duration) || e.duration < 0) throw RangeError("timeline duration must be finite and non-negative");
	let n = new Set(t.edges.map((e) => e.id)), r = /* @__PURE__ */ new Set([...t.nodes.map((e) => e.id), ...n]), i = /* @__PURE__ */ new Set();
	e.tracks.forEach((t, a) => {
		if (i.has(t.id)) throw Error(`duplicate timeline track id: ${t.id}`);
		if (i.add(t.id), !r.has(t.target)) throw Error(`timeline track ${t.id} targets missing scene id ${t.target}`);
		if (t.keyframes.length === 0) throw Error(`timeline track ${t.id} must contain a keyframe`);
		let o = n.has(t.target);
		if (o && !Oi.has(t.property)) throw Error(`timeline track ${t.id}: ${t.property} cannot target edge ${t.target} (edges accept opacity, progress/edgeReveal, highlight, flow)`);
		if (!o && !ki.has(t.property)) throw Error(`timeline track ${t.id}: ${t.property} cannot target node ${t.target} (nodes accept opacity, translateX/Y, scale, progress, highlight, revealX/Y)`);
		let s = -Infinity;
		t.keyframes.forEach((n, r) => {
			if (!Number.isFinite(n.time) || n.time < 0 || n.time > e.duration) throw RangeError(`tracks[${a}].keyframes[${r}].time is outside the timeline`);
			if (!Number.isFinite(n.value)) throw RangeError(`tracks[${a}].keyframes[${r}].value must be finite`);
			if (n.time <= s) throw Error(`timeline track ${t.id} keyframes must have strictly increasing times`);
			s = n.time;
		});
	});
}
function ji(e, t) {
	let n = e.keyframes[0], r = e.keyframes[e.keyframes.length - 1];
	if (n === void 0 || r === void 0) throw Error(`timeline track ${e.id} has no keyframes`);
	if (t <= n.time) return n.value;
	if (t >= r.time) return r.value;
	for (let n = 1; n < e.keyframes.length; n += 1) {
		let r = e.keyframes[n], i = e.keyframes[n - 1];
		if (i !== void 0 && r !== void 0 && t <= r.time) {
			let e = (t - i.time) / (r.time - i.time);
			return i.value + (r.value - i.value) * Xe(r.easing, e);
		}
	}
	return r.value;
}
function Mi(e, t, n) {
	if (t.length === 0) return e;
	let r = { ...e.state };
	for (let i of t) {
		let t = ji(i, n);
		switch (i.property) {
			case "opacity":
				r.opacity = Di(t, 0, 1) * Di(e.state.opacity, 0, 1);
				break;
			case "progress":
				r.progress = Di(t, 0, 1);
				break;
			case "highlight":
				r.highlight = Math.max(Di(t, 0, 1), e.state.highlight ?? 0);
				break;
			case "translateX":
			case "translateY":
			case "scale":
				r[i.property] = t;
				break;
			case "revealX":
				r.revealX = Di(t, 0, 1);
				break;
			case "revealY":
				r.revealY = Di(t, 0, 1);
				break;
			case "edgeReveal":
			case "flow": throw Error(`${i.property} track ${i.id} cannot target node ${e.id}`);
		}
	}
	return {
		...e,
		state: r
	};
}
function Ni(e, t, n) {
	let r = { ...e.state };
	for (let i of t) {
		let t = ji(i, n);
		if (i.property === "opacity") r.opacity = Di(t, 0, 1) * Di(e.state.opacity, 0, 1);
		else if (i.property === "progress" || i.property === "edgeReveal") r.progress = Di(t, 0, 1);
		else if (i.property === "highlight") r.highlight = Math.max(Di(t, 0, 1), e.state.highlight ?? 0);
		else if (i.property === "flow") r.flow = Di(t, 0, 1);
		else throw Error(`${i.property} track ${i.id} cannot target edge ${e.id}`);
	}
	let i = Pi(e.metadata?.packetCount), a = Pi(e.metadata?.packetPeriod), o = e.samples !== void 0 && i > 0 && a > 0 ? dn(e.samples, i, a, n) : e.packets;
	return t.length === 0 && o === e.packets ? e : {
		...e,
		state: r,
		...o === void 0 ? {} : { packets: o }
	};
}
function Pi(e) {
	return typeof e == "number" && Number.isFinite(e) ? e : 0;
}
function Fi(e, t) {
	if (!Number.isFinite(t)) throw RangeError("seek time must be finite");
	let n = e.timeline ?? {
		duration: 0,
		tracks: []
	};
	Ai(n, e);
	let r = Di(t, 0, n.duration), i = /* @__PURE__ */ new Map();
	for (let e of n.tracks) {
		let t = i.get(e.target) ?? [];
		t.push(e), i.set(e.target, t);
	}
	return {
		...e,
		time: r,
		progress: n.duration === 0 ? 1 : r / n.duration,
		nodes: e.nodes.map((e) => Mi(e, i.get(e.id) ?? [], r)),
		edges: e.edges.map((e) => Ni(e, i.get(e.id) ?? [], r))
	};
}
//#endregion
//#region ../svg/dist/motifs.js
var H = (e, t) => ({
	tag: "path",
	attrs: { d: e },
	...t === void 0 ? {} : { fill: t }
}), U = (e, t, n, r) => ({
	tag: "circle",
	attrs: {
		cx: String(e),
		cy: String(t),
		r: String(n)
	},
	...r === void 0 ? {} : { fill: r }
}), W = (e, t, n, r, i = 1, a) => ({
	tag: "rect",
	attrs: {
		x: String(e),
		y: String(t),
		width: String(n),
		height: String(r),
		rx: String(i)
	},
	...a === void 0 ? {} : { fill: a }
}), Ii = (e, t, n, r) => ({
	tag: "line",
	attrs: {
		x1: String(e),
		y1: String(t),
		x2: String(n),
		y2: String(r)
	}
}), Li = {
	field: [
		U(0, 0, 10, "background"),
		U(0, 0, 6, "background"),
		U(0, 0, 2, "stroke")
	],
	graph: [
		H("M -8 7 L 0 -8 L 8 6 Z"),
		U(-8, 7, 2.6, "background"),
		U(0, -8, 2.6, "background"),
		U(8, 6, 2.6, "background")
	],
	boundary: [U(0, 0, 10, "background"), H("M -10 0 C -5 -6 5 6 10 0")],
	blocks: [
		W(-9, -9, 8, 8),
		W(1, -9, 8, 8),
		W(-9, 1, 8, 8),
		W(1, 1, 8, 8)
	],
	box: [H("M -9 -5 L 0 -10 L 9 -5 L 9 5 L 0 10 L -9 5 Z"), H("M -9 -5 L 0 0 L 9 -5 M 0 0 L 0 10")],
	cube: [H("M -9 -5 L 0 -10 L 9 -5 L 9 5 L 0 10 L -9 5 Z"), H("M -9 -5 L 0 0 L 9 -5 M 0 0 L 0 10")],
	sphere: [
		U(0, 0, 10, "background"),
		H("M -10 0 C -6 -4 6 -4 10 0 M -10 0 C -6 4 6 4 10 0"),
		H("M 0 -10 C -4 -6 -4 6 0 10")
	],
	brush: [H("M 4 -10 L 10 -4 L -1 7 L -7 1 Z"), H("M -7 1 C -9 5 -9 8 -11 11 C -6 10 -3 9 -1 7")],
	layers: [
		H("M -10 -3 L 0 -8 L 10 -3 L 0 2 Z"),
		H("M -10 2 L 0 7 L 10 2"),
		H("M -10 6 L 0 11 L 10 6")
	],
	palette: [
		H("M 0 -10 C -6 -10 -10 -6 -10 0 C -10 6 -6 10 0 10 C 3 10 4 8 4 6 C 4 4 6 3 8 3 C 10 3 10 1 10 -1 C 10 -6 6 -10 0 -10 Z"),
		U(-5, -3, 1.6, "stroke"),
		U(0, -6, 1.6, "stroke"),
		U(5, -3, 1.6, "stroke"),
		U(-5, 3, 1.6, "stroke")
	],
	gear: [U(0, 0, 4), H("M 0 -11 L 0 -7 M 0 7 L 0 11 M -11 0 L -7 0 M 7 0 L 11 0 M -7.8 -7.8 L -5 -5 M 5 5 L 7.8 7.8 M 7.8 -7.8 L 5 -5 M -5 5 L -7.8 7.8")],
	chip: [
		W(-7, -7, 14, 14, 2),
		W(-3, -3, 6, 6, 1),
		H("M -3 -11 L -3 -7 M 3 -11 L 3 -7 M -3 7 L -3 11 M 3 7 L 3 11 M -11 -3 L -7 -3 M -11 3 L -7 3 M 7 -3 L 11 -3 M 7 3 L 11 3")
	],
	world: [U(0, 0, 10, "background"), H("M -10 0 L 10 0 M 0 -10 C -5 -5 -5 5 0 10 M 0 -10 C 5 -5 5 5 0 10 M -8 -5 L 8 -5 M -8 5 L 8 5")],
	code: [H("M -4 -7 L -10 0 L -4 7 M 4 -7 L 10 0 L 4 7 M 2 -10 L -2 10")],
	mesh: [H("M -10 -6 L -2 -10 L 10 -4 L 2 0 Z"), H("M -10 -6 L -8 4 L 2 10 L 2 0 M 2 10 L 10 6 L 10 -4")],
	camera: [H("M -10 -5 L -4 -5 L -2 -8 L 2 -8 L 4 -5 L 10 -5 L 10 8 L -10 8 Z"), U(0, 1, 4)],
	film: [W(-10, -8, 20, 16, 2), H("M -6 -8 L -6 8 M 6 -8 L 6 8 M -10 -3 L -6 -3 M -10 3 L -6 3 M 6 -3 L 10 -3 M 6 3 L 10 3")],
	arrow: [H("M -10 0 L 8 0 M 2 -6 L 8 0 L 2 6")],
	bolt: [H("M 2 -11 L -7 2 L 0 2 L -2 11 L 7 -2 L 0 -2 Z")],
	filter: [H("M -10 -9 L 10 -9 L 2 1 L 2 9 L -2 11 L -2 1 Z")],
	grid: [H("M -10 -10 L 10 -10 L 10 10 L -10 10 Z M -3.3 -10 L -3.3 10 M 3.3 -10 L 3.3 10 M -10 -3.3 L 10 -3.3 M -10 3.3 L 10 3.3")],
	dots: [
		U(-6, -6, 1.8, "stroke"),
		U(0, -6, 1.8, "stroke"),
		U(6, -6, 1.8, "stroke"),
		U(-6, 0, 1.8, "stroke"),
		U(0, 0, 1.8, "stroke"),
		U(6, 0, 1.8, "stroke"),
		U(-6, 6, 1.8, "stroke"),
		U(0, 6, 1.8, "stroke"),
		U(6, 6, 1.8, "stroke")
	],
	wave: [H("M -11 0 C -8 -8 -4 -8 -1 0 C 2 8 6 8 11 0")],
	funnel: [H("M -10 -8 L 10 -8 L 3 0 L 3 9 L -3 9 L -3 0 Z")],
	plug: [H("M -4 -11 L -4 -5 M 4 -11 L 4 -5 M -8 -5 L 8 -5 L 8 0 C 8 5 4 8 0 8 C -4 8 -8 5 -8 0 Z M 0 8 L 0 12")],
	book: [H("M -10 -8 C -6 -10 -2 -9 0 -7 C 2 -9 6 -10 10 -8 L 10 8 C 6 6 2 7 0 9 C -2 7 -6 6 -10 8 Z M 0 -7 L 0 9")],
	terminal: [W(-10, -8, 20, 16, 2), H("M -6 -3 L -2 0 L -6 3 M 0 4 L 6 4")],
	tag: [H("M -10 -10 L 0 -10 L 10 0 L 0 10 L -10 0 Z"), U(-5, -5, 1.6, "stroke")],
	clock: [U(0, 0, 10, "background"), H("M 0 -6 L 0 0 L 5 3")],
	shield: [H("M 0 -11 L 9 -7 L 9 0 C 9 6 5 9 0 11 C -5 9 -9 6 -9 0 L -9 -7 Z"), H("M -4 0 L -1 3 L 5 -3")],
	compare: [H("M -10 -6 L -2 -6 M -10 0 L -2 0 M -10 6 L -2 6 M 2 -6 L 10 -6 M 2 0 L 10 0 M 2 6 L 10 6")],
	branch: [
		U(-6, -6, 2.5, "background"),
		U(-6, 6, 2.5, "background"),
		U(6, 0, 2.5, "background"),
		H("M -3.5 -6 C 0 -6 0 0 3.5 0 M -3.5 6 C 0 6 0 0 3.5 0")
	],
	merge: [
		U(6, -6, 2.5, "background"),
		U(6, 6, 2.5, "background"),
		U(-6, 0, 2.5, "background"),
		H("M 3.5 -6 C 0 -6 0 0 -3.5 0 M 3.5 6 C 0 6 0 0 -3.5 0")
	],
	ramp: [
		W(-10, 4, 5, 5, .5, "stroke"),
		W(-3.5, 0, 5, 9, .5, "stroke"),
		W(3, -5, 5, 14, .5, "stroke")
	],
	gradient: [
		W(-10, -5, 20, 10, 1),
		Ii(-6, -5, -6, 5),
		Ii(-2, -5, -2, 5),
		Ii(2, -5, 2, 5),
		Ii(6, -5, 6, 5)
	],
	dither: [
		W(-10, -10, 4, 4, 0, "stroke"),
		W(-2, -10, 4, 4, 0, "stroke"),
		W(6, -10, 4, 4, 0, "stroke"),
		W(-6, -6, 4, 4, 0, "stroke"),
		W(2, -6, 4, 4, 0, "stroke"),
		W(-10, -2, 4, 4, 0, "stroke"),
		W(-2, -2, 4, 4, 0, "stroke"),
		W(6, -2, 4, 4, 0, "stroke"),
		W(-6, 2, 4, 4, 0, "stroke"),
		W(2, 2, 4, 4, 0, "stroke"),
		W(-10, 6, 4, 4, 0, "stroke"),
		W(-2, 6, 4, 4, 0, "stroke"),
		W(6, 6, 4, 4, 0, "stroke")
	],
	target: [
		U(0, 0, 10, "background"),
		U(0, 0, 5.5, "background"),
		U(0, 0, 1.8, "stroke")
	],
	signal: [H("M -10 4 C -6 -6 -2 -6 0 0 C 2 6 6 6 10 -4"), H("M -10 8 L 10 8")],
	piston: [
		W(-8, -3, 16, 12, 1),
		H("M -3 -3 L -3 -10 L 3 -10 L 3 -3"),
		H("M -8 3 L 8 3")
	],
	circuit: [
		H("M -10 -6 L -4 -6 L -4 6 L 4 6 L 4 -6 L 10 -6"),
		U(-4, -6, 2, "stroke"),
		U(4, -6, 2, "stroke")
	],
	clockTick: [U(0, 0, 10, "background"), H("M 0 -10 L 0 -7 M 10 0 L 7 0 M 0 10 L 0 7 M -10 0 L -7 0 M 0 0 L 4 -4")],
	file: [H("M -7 -11 L 3 -11 L 8 -6 L 8 11 L -7 11 Z"), H("M 3 -11 L 3 -6 L 8 -6")],
	detect: [U(-2, -2, 7, "background"), H("M 3 3 L 10 10")],
	export: [H("M -8 2 L -8 10 L 8 10 L 8 2 M 0 -10 L 0 5 M -5 -5 L 0 -10 L 5 -5")],
	bridge: [H("M -11 4 L -8 4 C -6 -4 6 -4 8 4 L 11 4 M -6 4 L -6 9 M 6 4 L 6 9 M 0 -2 L 0 9 M -11 9 L 11 9")],
	languages: [H("M -10 -6 L -4 -6 L -4 6 M -10 6 L -4 6 M -10 0 L -6 0"), H("M 2 -6 L 8 -6 M 5 -6 L 5 6 M 2 6 L 8 6")],
	rust: [
		U(0, 0, 9),
		U(0, 0, 3.5),
		H("M 0 -12 L 0 -9 M 0 9 L 0 12 M -12 0 L -9 0 M 9 0 L 12 0 M -8.5 -8.5 L -6.4 -6.4 M 6.4 6.4 L 8.5 8.5 M 8.5 -8.5 L 6.4 -6.4 M -6.4 6.4 L -8.5 8.5")
	],
	texture: [W(-10, -10, 20, 20, 1), H("M -10 -3 L 10 -3 M -10 3 L 10 3 M -3 -10 L -3 10 M 3 -10 L 3 10")],
	lightbulb: [H("M -6 -2 C -6 -8 6 -8 6 -2 C 6 2 3 3 3 6 L -3 6 C -3 3 -6 2 -6 -2 Z"), H("M -3 9 L 3 9 M -2 12 L 2 12")],
	eye: [H("M -11 0 C -6 -8 6 -8 11 0 C 6 8 -6 8 -11 0 Z"), U(0, 0, 3.2, "stroke")],
	spark: [H("M 0 -11 L 2 -2 L 11 0 L 2 2 L 0 11 L -2 2 L -11 0 L -2 -2 Z")],
	timeline: [
		H("M -11 0 L 11 0"),
		U(-6, 0, 2.2, "stroke"),
		U(0, 0, 2.2, "stroke"),
		U(6, 0, 2.2, "stroke"),
		H("M -6 -2 L -6 -7 M 6 -2 L 6 -7 M 0 2 L 0 7")
	],
	diamond: [H("M 0 -10 L 10 0 L 0 10 L -10 0 Z")]
};
function Ri(e) {
	return Li[e] ?? Li.diamond ?? [];
}
Object.keys(Li);
//#endregion
//#region ../svg/dist/index.js
function zi(e, t = {}) {
	let n = e, r = Ma(t.precision, 0, 12, 3), i = Aa(n.width, 640), a = Aa(n.height, 360), o = Ea(n.nodes), s = Ea(n.edges), c = K(n.theme), l = q(c.accent, K(K(c.tokens).colors).accent) ?? "#2563eb", u = J(n.id) ?? "scene", d = Ba(t.idPrefix ?? `kineglyph-${u}`), f = J(n.root) !== void 0, p = t.title ?? q(n.label, n.title, K(n.accessibility).label), m = t.description ?? q(n.description, K(n.accessibility).description), h = o.some(Ta), g = [p && `${d}-title`, m && `${d}-description`].filter(Boolean).join(" "), _ = J(n.background), v = [
		["xmlns", t.includeXmlns === !1 ? void 0 : "http://www.w3.org/2000/svg"],
		["id", d],
		["class", za("kg-scene", t.className)],
		["viewBox", `0 0 ${Z(i, r)} ${Z(a, r)}`],
		["preserveAspectRatio", "xMidYMid meet"],
		["width", Z(i, r)],
		["height", Z(a, r)],
		["role", t.role ?? (h ? "group" : "img")],
		["aria-labelledby", g || void 0],
		["aria-label", g ? void 0 : "Kineglyph scene"],
		["data-kineglyph-scene", u],
		["data-layout", q(n.layoutName, n.layout)],
		["style", ya(c)]
	], y = {
		rootId: d,
		precision: r,
		accent: l,
		background: _ ?? q(c.background) ?? "transparent",
		markers: /* @__PURE__ */ new Map(),
		animateFlow: t.animateFlow !== !1,
		structured: f,
		nodesById: new Map(o.map((e, t) => [Sa(e, t), e])),
		enhancedEffects: t.effects === "enhanced"
	}, b = s.filter((e) => X(e.z, 0) <= 0), x = s.filter((e) => X(e.z, 0) > 0), S = (e, t) => e.length === 0 ? "" : G("g", [["class", t]], e.map((e, t) => na(e, t, y)).join("")), C = f ? G("g", [["class", "kg-nodes"]], ra(o, y)) : G("g", [["class", "kg-nodes"]], o.map((e, t) => fa(e, t, d, r)).join("")), w = S(b, "kg-edges"), T = S(x, "kg-edges kg-edges--above"), E = t.background === "none" || _ === void 0 || _ === "transparent" ? "" : G("rect", [
		["class", "kg-canvas"],
		["x", "0"],
		["y", "0"],
		["width", Z(i, r)],
		["height", Z(a, r)],
		["fill", _],
		["aria-hidden", "true"]
	], "");
	return G("svg", v, [
		p && G("title", [["id", `${d}-title`]], Va(p)),
		m && G("desc", [["id", `${d}-description`]], Va(m)),
		Ui(y, s),
		G("style", [], ba),
		E,
		w,
		C,
		T
	].filter(Boolean).join(""));
}
function Bi(e, t, n) {
	return `${e}-m-${t}-${Vi(n)}`;
}
function Vi(e) {
	return e.trim().toLowerCase().replace(/^#/, "").replace(/[^a-z0-9]+/g, "-").replace(/^-+|-+$/g, "") || "default";
}
function Hi(e, t, n) {
	if (t === "none") return "";
	let r = [
		["id", Bi(e, t, n)],
		["class", `kg-marker kg-marker--${t}`],
		["viewBox", "0 0 10 10"],
		["orient", "auto-start-reverse"],
		["markerUnits", "strokeWidth"],
		["data-marker-kind", t]
	];
	switch (t) {
		case "arrow": return G("marker", [
			...r,
			["refX", "8.5"],
			["refY", "5"],
			["markerWidth", "7"],
			["markerHeight", "7"]
		], G("path", [
			["d", "M 1.5 1.5 L 8.5 5 L 1.5 8.5"],
			["fill", "none"],
			["stroke", n],
			["stroke-width", "1.7"],
			["stroke-linecap", "round"],
			["stroke-linejoin", "round"]
		], ""));
		case "triangle": return G("marker", [
			...r,
			["refX", "9"],
			["refY", "5"],
			["markerWidth", "6"],
			["markerHeight", "6"]
		], G("path", [
			["d", "M 0.5 0.5 L 9.5 5 L 0.5 9.5 z"],
			["fill", n],
			["stroke", "none"]
		], ""));
		case "dot": return G("marker", [
			...r,
			["refX", "5"],
			["refY", "5"],
			["markerWidth", "5"],
			["markerHeight", "5"]
		], G("circle", [
			["cx", "5"],
			["cy", "5"],
			["r", "3.4"],
			["fill", n],
			["stroke", "none"]
		], ""));
		case "diamond": return G("marker", [
			...r,
			["refX", "9"],
			["refY", "5"],
			["markerWidth", "7"],
			["markerHeight", "7"]
		], G("path", [
			["d", "M 5 0.8 L 9.2 5 L 5 9.2 L 0.8 5 z"],
			["fill", n],
			["stroke", "none"]
		], ""));
		case "bar": return G("marker", [
			...r,
			["refX", "5"],
			["refY", "5"],
			["markerWidth", "5"],
			["markerHeight", "5"]
		], G("path", [
			["d", "M 5 0.5 L 5 9.5"],
			["fill", "none"],
			["stroke", n],
			["stroke-width", "1.8"],
			["stroke-linecap", "butt"]
		], ""));
	}
}
function Ui(e, t) {
	let n = [];
	for (let n of t) {
		let t = $i(n, e.accent), r = Zi(n.head, n.directed !== !1 && n.markerEnd !== !1 && n.arrow !== !1 ? "arrow" : "none"), i = Zi(n.tail, "none");
		for (let n of [r, i]) {
			if (n === "none") continue;
			let r = Bi(e.rootId, n, t.stroke);
			e.markers.has(r) || e.markers.set(r, Hi(e.rootId, n, t.stroke));
		}
	}
	n.push(...[...e.markers.entries()].sort(([e], [t]) => e < t ? -1 : +(e > t)).map(([, e]) => e));
	let r = [...e.nodesById.entries()].map(([t, n]) => Ki(t, K(n.appearance).fill, e)).filter((e) => e.length > 0).sort();
	n.push(...r);
	let i = [...e.nodesById.entries()].map(([t, n]) => Yi(t, K(n.appearance), e)).filter((e) => e.length > 0).sort();
	return n.push(...i), n.length === 0 ? "" : G("defs", [], n.join(""));
}
function Wi(e, t) {
	return `${e}-paint-${Ba(t)}-fill`;
}
function Gi(e, t, n) {
	if (typeof e == "string") return e;
	let r = J(K(e).type);
	return r === "linear-gradient" || r === "radial-gradient" ? `url(#${Wi(n.rootId, t)})` : "none";
}
function Ki(e, t, n) {
	let r = K(t), i = J(r.type);
	if (i !== "linear-gradient" && i !== "radial-gradient") return "";
	let a = Ea(r.stops).map((e) => G("stop", [
		["offset", `${Z(ja(Y(e.at), 0) * 100, n.precision)}%`],
		["stop-color", J(e.color) ?? n.accent],
		["stop-opacity", ja(Y(e.opacity), 1) === 1 ? void 0 : Z(ja(Y(e.opacity), 1), n.precision)]
	], "")).join(""), o = [
		["id", Wi(n.rootId, e)],
		["gradientUnits", "objectBoundingBox"],
		["spreadMethod", J(r.spread) ?? "pad"],
		["data-gradient-of", e]
	];
	if (i === "radial-gradient") {
		let e = Array.isArray(r.center) ? r.center : [], t = Array.isArray(r.focalPoint) ? r.focalPoint : [];
		return G("radialGradient", [
			...o,
			["cx", Z(ja(Y(e[0]), .5), n.precision)],
			["cy", Z(ja(Y(e[1]), .5), n.precision)],
			["fx", Z(ja(Y(t[0]), .5), n.precision)],
			["fy", Z(ja(Y(t[1]), .5), n.precision)],
			["r", Z(Math.max(0, X(r.radius, .5)), n.precision)]
		], a);
	}
	let s = X(r.angle, 0) * Math.PI / 180, c = Math.cos(s), l = Math.sin(s), u = .5 / Math.max(Math.abs(c), Math.abs(l), 1e-9);
	return G("linearGradient", [
		...o,
		["x1", Z(.5 - c * u, n.precision)],
		["y1", Z(.5 - l * u, n.precision)],
		["x2", Z(.5 + c * u, n.precision)],
		["y2", Z(.5 + l * u, n.precision)]
	], a);
}
function qi(e, t) {
	return `${e}-material-${Ba(t)}`;
}
function Ji(e) {
	return Ea(e.effects).flatMap((e) => J(e.type) === "shader" ? Ea(e.fallback) : [e]);
}
function Yi(e, t, n) {
	let r = Ji(t), i = Ea(t.effects).filter((e) => J(e.type) === "shader"), a = r.filter((e) => J(e.type) !== "backdrop");
	if (a.length === 0 && i.length === 0) return "";
	let o = [], s = [], c = "SourceGraphic", l = 0, u = (e) => `${e}-${l++}`;
	for (let e of a) {
		let t = J(e.type);
		if (t === "shadow") {
			let t = u(J(e.kind) === "inner" ? "inner-shadow" : "outer-shadow"), r = Math.max(0, X(e.blur, 16)) / 2, i = Array.isArray(e.offset) ? e.offset : [], a = X(i[0], 0), l = X(i[1], 8), d = J(e.color) ?? "#000000", f = ja(Y(e.opacity), .16);
			if (J(e.kind) === "inner") {
				let e = u("inner-blur"), i = u("inner-offset"), s = u("inner-cut"), p = u("inner-color");
				o.push(G("feGaussianBlur", [
					["in", "SourceAlpha"],
					["stdDeviation", Z(r, n.precision)],
					["result", e]
				], ""), G("feOffset", [
					["in", e],
					["dx", Z(a, n.precision)],
					["dy", Z(l, n.precision)],
					["result", i]
				], ""), G("feComposite", [
					["in", "SourceAlpha"],
					["in2", i],
					["operator", "out"],
					["result", s]
				], ""), G("feFlood", [
					["flood-color", d],
					["flood-opacity", Z(f, n.precision)],
					["result", p]
				], ""), G("feComposite", [
					["in", p],
					["in2", s],
					["operator", "in"],
					["result", t]
				], ""));
				let m = u("with-inner");
				o.push(G("feMerge", [["result", m]], G("feMergeNode", [["in", c]], "") + G("feMergeNode", [["in", t]], ""))), c = m;
			} else {
				let i = X(e.spread, 0), c = u("shadow-spread"), p = u("shadow-blur"), m = u("shadow-offset"), h = u("shadow-color"), g = Math.abs(i) > 1e-6 ? c : "SourceAlpha";
				Math.abs(i) > 1e-6 && o.push(G("feMorphology", [
					["in", "SourceAlpha"],
					["operator", i > 0 ? "dilate" : "erode"],
					["radius", Z(Math.abs(i), n.precision)],
					["result", c]
				], "")), o.push(G("feGaussianBlur", [
					["in", g],
					["stdDeviation", Z(r, n.precision)],
					["result", p]
				], ""), G("feOffset", [
					["in", p],
					["dx", Z(a, n.precision)],
					["dy", Z(l, n.precision)],
					["result", m]
				], ""), G("feFlood", [
					["flood-color", d],
					["flood-opacity", Z(f, n.precision)],
					["result", h]
				], ""), G("feComposite", [
					["in", h],
					["in2", m],
					["operator", "in"],
					["result", t]
				], "")), s.push(t);
			}
		} else if (t === "blur") {
			let t = u("blur");
			o.push(G("feGaussianBlur", [
				["in", c],
				["stdDeviation", Z(Math.max(0, X(e.radius, 0)) / 2, n.precision)],
				["result", t]
			], "")), c = t;
		} else if (t === "noise") {
			let t = u("noise"), r = u("noise-tone"), i = u("noise-alpha"), a = u("with-noise"), s = ja(Y(e.amount), .03), l = Math.max(.01, X(e.scale, .8));
			o.push(G("feTurbulence", [
				["type", "fractalNoise"],
				["baseFrequency", Z(.012 + l * .018, n.precision)],
				["numOctaves", "3"],
				["seed", String(Math.round(X(e.seed, 1)))],
				["result", t]
			], ""));
			let d = e.monochrome === !1 ? t : r;
			e.monochrome !== !1 && o.push(G("feColorMatrix", [
				["in", t],
				["type", "saturate"],
				["values", "0"],
				["result", r]
			], "")), o.push(G("feComponentTransfer", [["in", d], ["result", i]], G("feFuncA", [["type", "linear"], ["slope", Z(s, n.precision)]], "")), G("feBlend", [
				["in", c],
				["in2", i],
				["mode", e.monochrome === !1 ? "screen" : "soft-light"],
				["result", a]
			], "")), c = a;
		}
	}
	for (let e of i) {
		if (J(e.name) !== "liquid") continue;
		let t = u("liquid-field"), r = u("liquid"), i = K(e.uniforms);
		o.push(G("feTurbulence", [
			["type", "turbulence"],
			["baseFrequency", Z(X(i.frequency, .018), n.precision)],
			["numOctaves", "2"],
			["seed", String(Math.round(X(i.seed, 23)))],
			["result", t]
		], ""), G("feDisplacementMap", [
			["in", c],
			["in2", t],
			["scale", Z(X(i.strength, 5), n.precision)],
			["xChannelSelector", "R"],
			["yChannelSelector", "G"],
			["result", r]
		], "")), c = r;
	}
	if (s.length > 0) {
		let e = u("material-output");
		o.push(G("feMerge", [["result", e]], s.map((e) => G("feMergeNode", [["in", e]], "")).join("") + G("feMergeNode", [["in", c]], "")));
	}
	return G("filter", [
		["id", qi(n.rootId, e)],
		["x", "-50%"],
		["y", "-50%"],
		["width", "200%"],
		["height", "200%"],
		["color-interpolation-filters", "sRGB"],
		["data-material-filter-of", e]
	], o.join(""));
}
function Xi(e, t, n) {
	let r = Ea(e.effects);
	if (r.length === 0 && J(e.blendMode) === void 0) return [];
	let i = Ji(e), a = i.some((e) => J(e.type) !== "backdrop"), o = r.filter((e) => J(e.type) === "shader").map((e) => J(e.name)).filter((e) => e !== void 0), s = r.find((e) => J(e.type) === "shader"), c = i.find((e) => J(e.type) === "backdrop"), l = [], u = J(e.blendMode);
	if (u !== void 0 && u !== "normal" && l.push(`mix-blend-mode:${u}`), n.enhancedEffects && c !== void 0) {
		let e = Math.max(0, X(c.blur, 16)), t = Math.max(0, X(c.saturation, 1)), r = Math.max(0, X(c.brightness, 1)), i = `blur(${Z(e, n.precision)}px) saturate(${Z(t, n.precision)}) brightness(${Z(r, n.precision)})`;
		l.push(`backdrop-filter:${i}`, `-webkit-backdrop-filter:${i}`);
	}
	return [
		["filter", a || o.includes("liquid") ? `url(#${qi(n.rootId, t)})` : void 0],
		["style", l.length === 0 ? void 0 : l.join(";")],
		["data-material-effects", r.map((e) => J(e.type)).filter(Boolean).join(" ")],
		["data-shader", o.length === 0 ? void 0 : o.join(" ")],
		["data-shader-uniforms", s === void 0 ? void 0 : JSON.stringify(K(s.uniforms))]
	];
}
function Zi(e, t) {
	return e === "none" || e === "arrow" || e === "triangle" || e === "dot" || e === "diamond" || e === "bar" ? e : t;
}
function Qi(e, t, n, r) {
	let i = Math.max(0, Math.min(1, n));
	return i <= 0 ? {
		stroke: e,
		strokeWidth: r
	} : {
		stroke: Kt(e, t, i * .85),
		strokeWidth: r + i * 1.5
	};
}
function $i(e, t) {
	let n = Oa(K(e.style), K(e.appearance)), r = K(e.state), i = q(n.stroke, e.color) ?? "#64748b", a = X(n.strokeWidth, 2), o = ja(Y(r.highlight), 0), s = Qi(i, t, o, a);
	return {
		stroke: s.stroke,
		strokeWidth: s.strokeWidth,
		highlight: o
	};
}
function ea(e, t, n, r, i = 3) {
	let a = Math.max(0, Math.min(1, r)), o = Math.max(.5, t);
	if (e === "solid") return a < 1 ? {
		dasharray: `${Z(a, i)} 1`,
		pathLength: "1",
		linecap: void 0
	} : {
		dasharray: void 0,
		pathLength: void 0,
		linecap: void 0
	};
	let s = e === "dotted" ? .01 : Math.max(5, o * 3), c = e === "dotted" ? Math.max(4, o * 2.4) : Math.max(4, o * 2.2), l = e === "dotted" ? "round" : void 0;
	if (a >= 1) return {
		dasharray: `${Z(s, i)} ${Z(c, i)}`,
		pathLength: void 0,
		linecap: l
	};
	let u = Math.max(1, n), d = s / u, f = c / u, p = [], m = 0, h = 0;
	for (; m < a && h < 400;) {
		h += 1;
		let e = Math.min(d, a - m);
		if (p.push(e), m += e, m >= a) break;
		let t = Math.min(f, a - m);
		p.push(t), m += t;
	}
	return p.length % 2 == 1 ? p.push(1) : p.push(0, 1), {
		dasharray: p.map((e) => Z(e, i)).join(" "),
		pathLength: "1",
		linecap: l
	};
}
function ta(e, t, n, r, i = 3) {
	return ea(e, Math.max(.5, t), n, r, i);
}
function na(e, t, n) {
	let { rootId: r, precision: i } = n, a = J(e.id) ?? `edge-${t + 1}`, o = J(e.from) ?? J(K(e.source).id), s = J(e.to) ?? J(K(e.target).id), c = va(e, "start", o === void 0 ? void 0 : n.nodesById.get(o)), l = va(e, "end", s === void 0 ? void 0 : n.nodesById.get(s)), u = q(e.path, K(e.path).d, e.d) ?? `M ${Z(c.x, i)} ${Z(c.y, i)} L ${Z(l.x, i)} ${Z(l.y, i)}`, d = Oa(K(e.style), K(e.appearance)), f = K(e.state), p = ja(Y(e.progress, f.progress), 1), m = ja(Y(e.opacity, f.opacity, d.opacity), 1), h = e.hidden === !0, g = $i(e, n.accent), _ = Zi(e.head, e.directed !== !1 && e.markerEnd !== !1 && e.arrow !== !1 ? "arrow" : "none"), v = Zi(e.tail, "none"), y = e.dash === "dashed" || e.dash === "dotted" || e.dash === "flow" ? e.dash : "solid", b = X(e.length, Math.hypot(l.x - c.x, l.y - c.y)), x = ea(y, g.strokeWidth, b, p, i), S = J(e.description), C = q(e.label, e.title), w = Ea(e.labels), T = Ea(e.packets), E = ja(Y(f.flow), +(T.length > 0)), D = G("path", [
		["id", `${r}-${Ba(a)}`],
		["class", za("kg-edge", `kg-edge--${y}`, y === "flow" && p >= 1 && "kg-edge--flowing", J(e.className))],
		["d", u],
		["fill", "none"],
		["stroke", g.stroke],
		["stroke-width", Z(g.strokeWidth, i)],
		["stroke-linecap", x.linecap ?? q(d.strokeLinecap, d.linecap) ?? "round"],
		["stroke-linejoin", "round"],
		["opacity", m === 1 ? void 0 : Z(m, i)],
		["pathLength", x.pathLength],
		["stroke-dasharray", x.dasharray],
		["marker-end", _ !== "none" && p >= 1 ? `url(#${Bi(r, _, g.stroke)})` : void 0],
		["marker-start", v !== "none" && p > 0 ? `url(#${Bi(r, v, g.stroke)})` : void 0],
		["data-edge-id", a],
		["data-kineglyph-edge", a],
		["data-from", o],
		["data-to", s],
		["data-progress", Z(p, i)],
		["data-length", Z(b, i)],
		["data-dash", y],
		["data-head", _],
		["data-tail", v],
		["data-base-stroke", q(d.stroke, e.color)],
		["data-base-width", Na(d.strokeWidth, i)],
		["data-highlight", g.highlight > 0 ? Z(g.highlight, i) : void 0]
	], ""), O = [];
	if ((S !== void 0 || C !== void 0 || w.length > 0) && O.push(G("path", [
		["class", "kg-edge-hit"],
		["d", u],
		["fill", "none"],
		["stroke", "transparent"],
		["stroke-width", Z(Math.max(14, g.strokeWidth + 10), i)],
		["pointer-events", "stroke"],
		["data-edge-hit", a]
	], "")), O.push(D), w.length > 0) for (let e of w) {
		if (e.hidden === !0) continue;
		let t = J(e.text) ?? "", r = X(e.x, (c.x + l.x) / 2), a = X(e.y, (c.y + l.y) / 2), o = X(e.width, 0), s = X(e.height, 0), u = X(e.fontSize, 12);
		O.push(G("g", [
			["class", "kg-edge-label"],
			["data-edge-label", J(e.id)],
			["opacity", m === 1 ? void 0 : Z(m, i)]
		], G("rect", [
			["class", "kg-edge-label-halo"],
			["x", Z(r - o / 2, i)],
			["y", Z(a - s / 2, i)],
			["width", Z(o, i)],
			["height", Z(s, i)],
			["rx", Z(Math.min(6, s / 2), i)],
			["fill", n.background === "transparent" ? "none" : n.background],
			["fill-opacity", "0.9"]
		], "") + G("text", [
			["class", "kg-edge-label-text"],
			["x", Z(r, i)],
			["y", Z(a + u * .35, i)],
			["text-anchor", "middle"],
			["font-family", J(e.fontFamily)],
			["font-size", Z(u, i)],
			["font-weight", Na(e.fontWeight, i)],
			["fill", J(e.color)],
			["textLength", Z(Math.max(.1, o - 10), i)],
			["lengthAdjust", "spacingAndGlyphs"]
		], Va(t))));
	}
	else if (C !== void 0) {
		let e = {
			x: (c.x + l.x) / 2,
			y: (c.y + l.y) / 2
		};
		O.push(G("text", [
			["class", "kg-edge-label"],
			["x", Z(e.x, i)],
			["y", Z(e.y, i)],
			["text-anchor", "middle"],
			["dominant-baseline", "central"]
		], Va(C)));
	}
	if (T.length > 0) {
		let t = X(e.packetSize, Math.max(3, g.strokeWidth * 1.6)), r = J(e.packetColor) ?? g.stroke;
		T.forEach((e, o) => {
			O.push(G("circle", [
				["class", "kg-edge-packet"],
				["cx", Z(X(e.x, c.x), i)],
				["cy", Z(X(e.y, c.y), i)],
				["r", Z(t, i)],
				["fill", r],
				["stroke", n.background === "transparent" ? void 0 : n.background],
				["stroke-width", n.background === "transparent" ? void 0 : "1"],
				["opacity", Z(E * m, i)],
				["data-edge-packet", a],
				["data-packet-index", String(o)]
			], ""));
		});
	}
	return G("g", [
		["class", za("kg-edge-group", g.highlight > 0 && "kg-edge-group--highlight")],
		["data-edge-group", a],
		["role", S === void 0 ? void 0 : "img"],
		["aria-label", S],
		["aria-hidden", S === void 0 ? "true" : void 0],
		["display", h ? "none" : void 0],
		["data-hidden", h ? "true" : void 0]
	], O.join(""));
}
function ra(e, t) {
	let n = /* @__PURE__ */ new Map();
	e.forEach((e, t) => {
		let r = J(e.parent), i = n.get(r) ?? [];
		i.push({
			node: e,
			index: t
		}), n.set(r, i);
	});
	let r = (e) => (n.get(e) ?? []).slice().sort((e, t) => X(e.node.z, 0) - X(t.node.z, 0) || e.index - t.index).map((e) => e.node), i = new Set(e.map((e, t) => Sa(e, t))), a = e.filter((e) => {
		let t = J(e.parent);
		return t === void 0 || !i.has(t);
	}), o = new Set(a.map((e, t) => Sa(e, t))), s = (e, n) => {
		let i = n || e.focusGroup === !0;
		return ia(e, r(Sa(e, 0)).map((e) => s(e, i)).join(""), t, n);
	};
	return r(void 0).filter((e) => o.has(Sa(e, 0))).map((e) => s(e, !1)).join("");
}
function ia(e, t, n, r = !1) {
	let { rootId: i, precision: a } = n, o = Sa(e, 0), s = `${i}-node-${Ba(o)}`, c = Ca(e), l = q(e.label), u = q(e.description), d = Ta(e), f = e.focusGroup === !0, p = (ka(e.focusable) ?? d) || f, m = K(e.state), h = K(e.appearance), g = ja(Y(m.opacity), 1), _ = ja(Y(m.progress), 1), v = ja(Y(m.highlight), 0), y = X(m.translateX, 0), b = X(m.translateY, 0), x = Math.max(0, X(m.scale, 1)), S = wa(e), C = e.hidden === !0, w = oa(S, y, b, x, a), T = ja(Y(m.revealX), 1), E = ja(Y(m.revealY), 1), D = J(e.revealAnchor), O = D !== void 0 || m.revealX !== void 0 || m.revealY !== void 0, k = d || l !== void 0 && l.length > 0 && c !== "text" && c !== "badge" && c !== "callout", A = k ? [l && `${s}-title`, u && `${s}-description`].filter(Boolean).join(" ") : "", j = K(e.metadata), M = `${s}-clip`, N = [
		["id", s],
		["class", za("kg-node", `kg-node--${Ba(c)}`, d && "kg-node--interactive", v > 0 && "kg-node--highlight")],
		["role", d ? "button" : f ? "group" : k && A ? c === "image" ? "img" : "group" : void 0],
		["tabindex", p ? r && !f ? "-1" : "0" : void 0],
		["data-focus-group", f ? "true" : void 0],
		["focusable", p ? "true" : void 0],
		["aria-labelledby", A || void 0],
		["transform", w || void 0],
		["opacity", g === 1 ? void 0 : Z(g, a)],
		["display", C ? "none" : void 0],
		["data-node-id", o],
		["data-kineglyph-node", o],
		["data-kind", c],
		["data-interactive", d ? "true" : void 0],
		["data-activate", J(e.onActivate)],
		["data-hidden", C ? "true" : void 0],
		["data-progress", Z(_, a)],
		["data-highlight", v > 0 ? Z(v, a) : void 0],
		["style", `--kg-progress:${Z(_, a)};--kg-highlight:${Z(v, a)}`],
		...xa(j)
	], P = k ? [l && G("title", [["id", `${s}-title`]], Va(l)), u && G("desc", [["id", `${s}-description`]], Va(u))].filter(Boolean).join("") : "", ee = e.clip === !0, te = ee ? G("clipPath", [["id", M]], G("rect", [
		["x", Z(S.x, a)],
		["y", Z(S.y, a)],
		["width", Z(S.width, a)],
		["height", Z(S.height, a)],
		["rx", Na(h.radius, a)]
	], "")) : "", F = ca(e, c, S, h, v, _, n), ne = ee ? G("g", [["clip-path", `url(#${M})`]], t) : t;
	if (!O) return G("g", N, P + te + F + ne);
	let re = `${s}-reveal`, ie = sa(S, T, E, D), ae = G("clipPath", [["id", re]], G("rect", [
		["x", Z(ie.x, a)],
		["y", Z(ie.y, a)],
		["width", Z(Math.max(0, ie.width), a)],
		["height", Z(Math.max(0, ie.height), a)],
		["data-reveal-clip", o]
	], ""));
	return G("g", [
		...N,
		["data-reveal-x", Z(T, a)],
		["data-reveal-y", Z(E, a)]
	], P + te + ae + G("g", [["clip-path", `url(#${re})`]], F + ne));
}
function aa(e, t, n, r) {
	let i = e.x + e.width / 2, a = e.y + e.height / 2;
	return {
		tx: t + i * (1 - r),
		ty: n + a * (1 - r),
		scale: r
	};
}
function oa(e, t, n, r, i) {
	let { tx: a, ty: o } = aa(e, t, n, r);
	return [a !== 0 || o !== 0 ? `translate(${Z(a, i)} ${Z(o, i)})` : "", r === 1 ? "" : `scale(${Z(r, i)})`].filter(Boolean).join(" ");
}
function sa(e, t, n, r) {
	let i = Math.max(0, Math.min(1, t)), a = Math.max(0, Math.min(1, n)), o = e.width * i, s = e.height * a, c = r === "right" ? e.x + e.width - o : e.x, l = r === "top" ? e.y : e.y + e.height - s;
	return {
		x: i < 1 ? c : e.x,
		y: a < 1 ? l : e.y,
		width: i < 1 ? o : e.width,
		height: a < 1 ? s : e.height
	};
}
function ca(e, t, n, r, i, a, o) {
	let { precision: s, accent: c } = o, { x: l, y: u, width: d, height: f } = n, p = Sa(e, 0), m = Gi(r.fill, p, o), h = J(r.stroke) ?? "none", g = X(r.strokeWidth, 1), _ = Qi(h === "none" ? c : h, c, i, h === "none" ? 0 : g), v = h === "none" && i <= 0 ? "none" : _.stroke, y = h === "none" && i <= 0 ? 0 : _.strokeWidth, b = i > 0 && m !== "none" && !m.startsWith("url(") ? Kt(m, c, i * .12) : m, x = J(r.dash), S = x === "dashed" ? `${Z(Math.max(4, y * 3), s)} ${Z(Math.max(3, y * 2), s)}` : x === "dotted" ? `0.01 ${Z(Math.max(3, y * 2.4), s)}` : void 0, C = [
		["class", "kg-node-shape"],
		["data-shape-of", p],
		["fill", b],
		["stroke", v],
		["stroke-width", y > 0 ? Z(y, s) : void 0],
		["stroke-dasharray", S],
		["stroke-linecap", x === "dotted" ? "round" : void 0],
		["fill-opacity", Na(r.opacity, s)],
		["stroke-opacity", Na(r.opacity, s)],
		...Xi(r, p, o)
	];
	switch (t) {
		case "group":
		case "rect": return b === "none" && v === "none" ? "" : G("rect", [
			...C,
			["x", Z(l, s)],
			["y", Z(u, s)],
			["width", Z(d, s)],
			["height", Z(f, s)],
			["rx", Na(Math.min(X(r.radius, 0), d / 2, f / 2), s)]
		], "");
		case "circle": {
			let e = l + d / 2, t = u + f / 2;
			return Math.abs(d - f) < 1e-6 ? G("circle", [
				...C,
				["cx", Z(e, s)],
				["cy", Z(t, s)],
				["r", Z(d / 2, s)]
			], "") : G("ellipse", [
				...C,
				["cx", Z(e, s)],
				["cy", Z(t, s)],
				["rx", Z(d / 2, s)],
				["ry", Z(f / 2, s)]
			], "");
		}
		case "text": return ua(K(e.text), "kg-text", s, a, p);
		case "badge": return G("rect", [
			...C,
			["x", Z(l, s)],
			["y", Z(u, s)],
			["width", Z(d, s)],
			["height", Z(f, s)],
			["rx", Z(Math.min(f / 2, X(r.radius, f / 2)), s)]
		], "") + ua(K(e.text), "kg-text kg-badge-text", s, 1);
		case "icon": {
			let t = K(e.icon), n = J(t.name) ?? "diamond", r = X(t.size, Math.min(d, f)), a = i > 0 ? Kt(J(t.color) ?? c, c, i * .6) : J(t.color) ?? c, p = J(t.background) ?? "none";
			return da(n, l + d / 2, u + f / 2, r, a, p, o.background, s);
		}
		case "path": {
			let t = K(e.path), n = K(t.viewBox), i = Aa(n.width, 24), c = Aa(n.height, 24), m = Math.max(1e-6, Math.min(d / i, f / c)), h = l + (d - i * m) / 2, g = u + (f - c * m) / 2, _ = y > 0 ? y / m : 0, S = X(t.length, Math.hypot(i, c)), C = x === "dashed" || x === "dotted" ? x : "solid", w = ta(C, _, S, a, s), T = J(r.lineCap) ?? w.linecap ?? "round";
			return G("path", [
				["class", "kg-node-shape kg-path"],
				["data-shape-of", p],
				["d", J(t.d) ?? ""],
				["fill", b],
				["stroke", v],
				["stroke-width", _ > 0 ? Z(_, s) : void 0],
				["fill-opacity", Na(r.opacity, s)],
				["stroke-opacity", Na(r.opacity, s)],
				["stroke-linecap", T],
				["stroke-linejoin", "round"],
				["pathLength", w.pathLength],
				["stroke-dasharray", w.dasharray],
				["data-path-length", Z(S, s)],
				["data-dash", C],
				...Xi(r, p, o),
				["transform", `translate(${Z(h, s)} ${Z(g, s)}) scale(${Z(m, s)})`]
			], "");
		}
		case "image": {
			let t = K(e.image), n = J(t.fit) ?? "contain", i = n === "cover" ? "xMidYMid slice" : n === "fill" ? "none" : "xMidYMid meet", a = X(r.radius, 0), c = `${o.rootId}-node-${Ba(Sa(e, 0))}-image-clip`, m = G("image", [
				["class", "kg-image"],
				["href", J(t.href)],
				["x", Z(l, s)],
				["y", Z(u, s)],
				["width", Z(d, s)],
				["height", Z(f, s)],
				["preserveAspectRatio", i],
				["clip-path", a > 0 ? `url(#${c})` : void 0],
				["data-live", t.live === !0 ? "true" : void 0],
				...Xi(r, p, o)
			], ""), h = a > 0 ? G("clipPath", [["id", c]], G("rect", [
				["x", Z(l, s)],
				["y", Z(u, s)],
				["width", Z(d, s)],
				["height", Z(f, s)],
				["rx", Z(a, s)]
			], "")) : "", g = J(t.alt);
			return h + (g === void 0 ? m : G("g", [["role", "img"], ["aria-label", g]], m));
		}
		case "legend": {
			let t = K(e.legend), n = Ea(t.items), r = K(t.text);
			return n.map((e) => {
				let t = K(e.box), n = X(t.x, l), i = X(t.y, u), a = X(t.height, 16), o = J(e.shape) ?? "square", d = J(e.swatch) ?? c, f = i + a / 2, p = o === "circle" ? G("circle", [
					["cx", Z(n + 6, s)],
					["cy", Z(f, s)],
					["r", "5"],
					["fill", d]
				], "") : o === "line" || o === "dashed" ? G("path", [
					["d", `M ${Z(n, s)} ${Z(f, s)} L ${Z(n + 12, s)} ${Z(f, s)}`],
					["stroke", d],
					["stroke-width", "2"],
					["stroke-linecap", "round"],
					["stroke-dasharray", o === "dashed" ? "3 3" : void 0],
					["fill", "none"]
				], "") : G("rect", [
					["x", Z(n, s)],
					["y", Z(f - 5, s)],
					["width", "10"],
					["height", "10"],
					["rx", "2"],
					["fill", d]
				], ""), m = X(r.fontSize, 12), h = G("text", [
					["class", "kg-text kg-legend-text"],
					["x", Z(n + 19, s)],
					["y", Z(f + m * .35, s)],
					["font-family", J(r.fontFamily)],
					["font-size", Z(m, s)],
					["font-weight", Na(r.fontWeight, s)],
					["fill", J(r.color)]
				], Va(J(e.label) ?? ""));
				return G("g", [["class", "kg-legend-item"], ["data-legend-item", J(e.id)]], p + h);
			}).join("");
		}
		case "callout": {
			let t = K(e.callout), n = K(t.body), i = K(t.tip), a = J(t.pointer) ?? "none", o = X(n.x, l), c = X(n.y, u), p = X(n.width, d), m = X(n.height, f), h = la(o, c, p, m, Math.min(X(r.radius, 8), p / 2, m / 2), a, X(i.x, o), X(i.y, c), s);
			return G("path", [
				["class", "kg-node-shape kg-callout"],
				...C,
				["d", h]
			], "") + ua(K(e.text), "kg-text kg-callout-text", s, 1);
		}
		default: return "";
	}
}
function la(e, t, n, r, i, a, o, s, c) {
	let l = (e) => Z(e, c), u = [];
	if (u.push(`M ${l(e + i)} ${l(t)}`), a === "up") {
		let r = Math.min(Math.max(o, e + i + 7), e + n - i - 7);
		u.push(`L ${l(r - 7)} ${l(t)} L ${l(r)} ${l(t - 8)} L ${l(r + 7)} ${l(t)}`);
	}
	if (u.push(`L ${l(e + n - i)} ${l(t)} Q ${l(e + n)} ${l(t)} ${l(e + n)} ${l(t + i)}`), a === "right") {
		let a = Math.min(Math.max(s, t + i + 7), t + r - i - 7);
		u.push(`L ${l(e + n)} ${l(a - 7)} L ${l(e + n + 8)} ${l(a)} L ${l(e + n)} ${l(a + 7)}`);
	}
	if (u.push(`L ${l(e + n)} ${l(t + r - i)} Q ${l(e + n)} ${l(t + r)} ${l(e + n - i)} ${l(t + r)}`), a === "down") {
		let a = Math.min(Math.max(o, e + i + 7), e + n - i - 7);
		u.push(`L ${l(a + 7)} ${l(t + r)} L ${l(a)} ${l(t + r + 8)} L ${l(a - 7)} ${l(t + r)}`);
	}
	if (u.push(`L ${l(e + i)} ${l(t + r)} Q ${l(e)} ${l(t + r)} ${l(e)} ${l(t + r - i)}`), a === "left") {
		let n = Math.min(Math.max(s, t + i + 7), t + r - i - 7);
		u.push(`L ${l(e)} ${l(n + 7)} L ${l(e - 8)} ${l(n)} L ${l(e)} ${l(n - 7)}`);
	}
	return u.push(`L ${l(e)} ${l(t + i)} Q ${l(e)} ${l(t)} ${l(e + i)} ${l(t)} Z`), u.join(" ");
}
function ua(e, t, n, r, i) {
	let a = Ea(e.lines), o = K(e.box);
	if (a.length === 0) return "";
	let s = X(o.x, 0), c = X(o.y, 0), l = X(o.width, 0), u = X(e.fontSize, 14), d = X(e.lineHeight, u * 1.4), f = J(e.align) ?? "start", p = f === "center" ? s + l / 2 : f === "end" ? s + l : s, m = X(e.letterSpacing, 0), h = r >= 1 ? a.length : Math.max(0, Math.round(a.length * r));
	return G("text", [
		["class", t],
		["data-text-of", i],
		["font-family", J(e.fontFamily)],
		["font-size", Z(u, n)],
		["font-weight", Na(e.fontWeight, n)],
		["letter-spacing", m === 0 ? void 0 : Z(m, n)],
		["fill", J(e.color)],
		["text-anchor", f === "center" ? "middle" : f === "end" ? "end" : void 0],
		["data-wrap-lines", String(a.length)],
		["data-max-width", Z(l, n)]
	], a.map((e, t) => {
		let r = X(e.width, 0), i = c + t * d + d / 2 + u * .35;
		return G("tspan", [
			["x", Z(p, n)],
			["y", Z(i, n)],
			["textLength", r > .5 ? Z(r, n) : void 0],
			["lengthAdjust", r > .5 ? "spacingAndGlyphs" : void 0],
			["data-line-width", Z(r, n)],
			["opacity", t < h ? void 0 : "0"]
		], Va(J(e.text) ?? ""));
	}).join(""));
}
function da(e, t, n, r, i, a, o, s) {
	let c = r / 24, l = Ri(e), u = (e) => {
		switch (e.fill) {
			case "stroke": return i;
			case "background": return a === "none" ? o === "transparent" ? "none" : o : a;
			default: return "none";
		}
	}, d = l.map((e) => G(e.tag, [
		...Object.entries(e.attrs),
		["fill", u(e)],
		["stroke", i],
		["stroke-width", Z(1.6 / Math.max(.35, Math.min(c, 1.4)), s)],
		["stroke-linecap", "round"],
		["stroke-linejoin", "round"]
	], "")).join(""), f = a === "none" ? "" : G("circle", [
		["cx", "0"],
		["cy", "0"],
		["r", Z(15, s)],
		["fill", a],
		["stroke", "none"]
	], "");
	return G("g", [
		["class", `kg-icon kg-icon--${Ba(e)}`],
		["transform", `translate(${Z(t, s)} ${Z(n, s)}) scale(${Z(c, s)})`],
		["aria-hidden", "true"],
		["data-icon", e]
	], f + d);
}
function fa(e, t, n, r) {
	let i = Sa(e, t), a = `${n}-node-${Ba(i)}`, o = q(e.label, e.title, e.name, K(e.accessibility).label), s = q(e.description, e.body, K(e.accessibility).description), c = Ta(e), l = ka(e.focusable) ?? ka(K(e.interaction).focusable) ?? c, u = Oa(K(e.style), K(e.appearance)), d = K(e.state), f = ja(Y(e.opacity, d.opacity, u.opacity), 1), p = ja(Y(e.progress, d.progress), 1), m = ja(Y(d.highlight), 0), h = X(d.translateX, 0), g = X(d.translateY, 0), _ = Math.max(0, X(d.scale, 1)), v = wa(e), y = oa(v, h, g, _, r), b = [o && `${a}-title`, s && `${a}-description`].filter(Boolean).join(" "), x = Oa(K(e.metadata), K(e.data)), S = `${a}-content-clip`;
	return G("g", [
		["id", a],
		["class", za("kg-node", `kg-node--${Ba(Ca(e))}`, J(e.className), c && "kg-node--interactive", m > 0 && "kg-node--highlight")],
		["role", q(e.role, K(e.accessibility).role) ?? (c ? "button" : b ? "group" : void 0)],
		["tabindex", l ? String(Ma(e.tabIndex, -1, 32767, 0)) : void 0],
		["focusable", l ? "true" : void 0],
		["aria-labelledby", b || void 0],
		["aria-label", b ? void 0 : c ? o ?? i : void 0],
		["aria-disabled", ka(e.disabled) === !0 || ka(d.disabled) === !0 ? "true" : void 0],
		["transform", y || void 0],
		["opacity", f === 1 ? void 0 : Z(f, r)],
		["data-node-id", i],
		["data-kineglyph-node", i],
		["data-interactive", c ? "true" : void 0],
		["data-progress", Z(p, r)],
		["data-highlight", m > 0 ? Z(m, r) : void 0],
		["style", `--kg-progress:${Z(p, r)}`],
		...xa(x)
	], [o && G("title", [["id", `${a}-title`]], Va(o)), s && G("desc", [["id", `${a}-description`]], Va(s))].filter(Boolean).join("") + G("clipPath", [["id", S]], G("rect", [
		["x", Z(v.x + 7, r)],
		["y", Z(v.y + 7, r)],
		["width", Z(Math.max(0, v.width - 14), r)],
		["height", Z(Math.max(0, v.height - 14), r)]
	], "")) + pa(e, u, p, n, r) + ma(e, o, S, r));
}
function pa(e, t, n, r, i) {
	let a = Ca(e), { x: o, y: s, width: c, height: l } = wa(e), u = [
		["class", "kg-node-shape"],
		["fill", Fa(q(t.fill, e.fill))],
		["stroke", Fa(q(t.stroke, e.stroke))],
		["stroke-width", Na(t.strokeWidth, i)],
		["stroke-linecap", q(t.strokeLinecap, t.linecap)],
		["stroke-linejoin", q(t.strokeLinejoin, t.linejoin)],
		["pathLength", n < 1 ? "1" : void 0],
		["stroke-dasharray", n < 1 ? `${Z(n, i)} 1` : void 0]
	];
	if (a === "circle") return G("circle", [
		...u,
		["cx", Z(X(e.cx, o + c / 2), i)],
		["cy", Z(X(e.cy, s + l / 2), i)],
		["r", Z(Math.max(0, X(e.r, Math.min(c, l) / 2)), i)]
	], "");
	if (a === "ellipse") return G("ellipse", [
		...u,
		["cx", Z(X(e.cx, o + c / 2), i)],
		["cy", Z(X(e.cy, s + l / 2), i)],
		["rx", Z(Math.max(0, X(e.rx, c / 2)), i)],
		["ry", Z(Math.max(0, X(e.ry, l / 2)), i)]
	], "");
	if (a === "line") return G("line", [
		...u,
		["x1", Z(X(e.x1, o), i)],
		["y1", Z(X(e.y1, s), i)],
		["x2", Z(X(e.x2, o + c), i)],
		["y2", Z(X(e.y2, s + l), i)]
	], "");
	if (a === "path") return G("path", [...u, ["d", q(e.d, K(e.path).d) ?? ""]], "");
	if (a === "polygon" || a === "polyline") return G(a, [...u, ["points", Ra(e.points, i)]], "");
	if (a === "text") {
		let n = q(e.text, e.value, e.label) ?? "";
		return G("text", [
			...u,
			["x", Z(o, i)],
			["y", Z(s, i)],
			["font-size", Na(t.fontSize, i)],
			["font-family", q(t.fontFamily)],
			["text-anchor", q(t.textAnchor)],
			["dominant-baseline", q(t.dominantBaseline)]
		], Va(n));
	}
	return a === "group" ? Ea(e.children).map((e, t) => fa(e, t, `${r}-group`, i)).join("") : G("rect", [
		...u,
		["x", Z(o, i)],
		["y", Z(s, i)],
		["width", Z(c, i)],
		["height", Z(l, i)],
		["rx", Ia(e.rx ?? t.radius, i, "radius")],
		["ry", Ia(e.ry ?? t.radius, i, "radius")]
	], "");
}
function ma(e, t, n, r) {
	if (Ca(e) === "text") return "";
	let i = q(e.body, e.subtitle, e.description), a = q(e.icon, K(e.metadata).icon, K(e.appearance).icon), o = q(K(e.metadata).motif);
	if (!t && !i && !a && !o) return "";
	let { x: s, y: c, width: l, height: u } = wa(e), d = a || o ? 28 : 0, f = s + 12 + d, p = Math.max(8, l - (f - s) - 12), m = t ? ha(t, p, {
		averageCharacterWidth: 7.1,
		maxLines: i ? 2 : 3
	}) : [], h = i ? ha(i, p, {
		averageCharacterWidth: 6.15,
		maxLines: 3
	}) : [], g = m.length > 0 && h.length > 0 ? 4 : 0, _ = m.length * 15 + g + h.length * 14, v = Math.max(c + 12, c + (u - _) / 2), y = [];
	if (o) y.push(_a(o, s + 12 + 9, c + u / 2, r));
	else if (a) {
		let e = s + 12 + 7, t = c + u / 2;
		y.push(G("circle", [
			["class", "kg-node-icon-bg"],
			["cx", Z(e, r)],
			["cy", Z(t, r)],
			["r", "8"],
			["aria-hidden", "true"],
			["data-icon", a]
		], "")), y.push(G("text", [
			["class", "kg-node-icon"],
			["x", Z(e, r)],
			["y", Z(t, r)],
			["text-anchor", "middle"],
			["dominant-baseline", "central"],
			["aria-hidden", "true"]
		], Va(a.slice(0, 1).toUpperCase())));
	}
	return m.length > 0 && y.push(ga("kg-node-label", m, f, v, 15, p, r)), h.length > 0 && y.push(ga("kg-node-body", h, f, v + m.length * 15 + g, 14, p, r)), G("g", [
		["class", "kg-node-content"],
		["pointer-events", "none"],
		["clip-path", `url(#${n})`]
	], y.join(""));
}
function ha(e, t, n) {
	let r = Math.max(.1, n.averageCharacterWidth), i = Math.max(1, Math.floor(Math.max(0, t) / r)), a = Math.max(1, Math.floor(n.maxLines)), o = e.trim().split(/\s+/).filter(Boolean), s = [], c = "", l = !1, u = () => {
		c.length > 0 && s.push(c), c = "";
	};
	for (let e of o) {
		let t = [];
		for (let n = 0; n < e.length; n += i) t.push(e.slice(n, n + i));
		for (let e of t) {
			let t = c.length === 0 ? e : `${c} ${e}`;
			if (t.length <= i) c = t;
			else {
				if (u(), s.length >= a) {
					l = !0;
					break;
				}
				c = e;
			}
		}
		if (l) break;
	}
	if (!l && c.length > 0 && u(), s.length > a && (s.length = a, l = !0), l && s.length > 0) {
		let e = s.length - 1;
		s[e] = `${(s[e] ?? "").slice(0, Math.max(0, i - 1)).trimEnd()}…`;
	}
	return s.map((e) => ({
		text: e,
		measuredWidth: Math.min(t, e.length * r)
	}));
}
function ga(e, t, n, r, i, a, o) {
	return G("text", [
		["class", e],
		["x", Z(n, o)],
		["y", Z(r, o)],
		["dominant-baseline", "hanging"],
		["data-wrap-lines", String(t.length)],
		["data-max-width", Z(a, o)]
	], t.map((e, t) => G("tspan", [
		["x", Z(n, o)],
		["y", Z(r + t * i, o)],
		["textLength", Z(Math.max(.1, e.measuredWidth), o)],
		["lengthAdjust", "spacingAndGlyphs"],
		["data-line-width", Z(e.measuredWidth, o)]
	], Va(e.text))).join(""));
}
function _a(e, t, n, r) {
	let i = Ri(e).map((e) => G(e.tag, [...Object.entries(e.attrs), ["class", e.fill === "background" ? "kg-motif-backed" : e.fill === "stroke" ? "kg-motif-solid" : void 0]], "")).join("");
	return G("g", [
		["class", `kg-node-motif kg-node-motif--${Ba(e)}`],
		["transform", `translate(${Z(t, r)} ${Z(n, r)}) scale(${Z(.8333333333333334, r)})`],
		["aria-hidden", "true"],
		["data-motif", e]
	], i);
}
function va(e, t, n) {
	let r = K(e[t]), i = K(e[t === "start" ? "source" : "target"]);
	if (r.x !== void 0 || r.y !== void 0) return {
		x: X(r.x, 0),
		y: X(r.y, 0)
	};
	if (i.x !== void 0 || i.y !== void 0) return {
		x: X(i.x, 0),
		y: X(i.y, 0)
	};
	if (n) {
		let e = wa(n, 0, 0);
		return {
			x: e.x + e.width / 2,
			y: e.y + e.height / 2
		};
	}
	let a = t === "start" ? "1" : "2";
	return {
		x: X(e[`x${a}`], 0),
		y: X(e[`y${a}`], 0)
	};
}
function ya(e) {
	let t = K(e.canvas), n = K(e.node), r = K(e.edge), i = K(e.text), a = K(e.semantic), o = K(e.tokens), s = Oa(K(o.colors), K(e.colors)), c = Oa(K(o.radii), K(e.radii)), l = K(Oa(K(o.typography), K(e.typography)).body), u = [
		["--kg-background", q(s.canvas, e.background, t.background, a.background) ?? "transparent"],
		["--kg-node-fill", q(s.surface, n.fill, e.nodeFill, a.surface) ?? "#ffffff"],
		["--kg-node-stroke", q(s.border, n.stroke, e.nodeStroke, a.foreground) ?? "#1f2937"],
		["--kg-edge-stroke", q(s.connector, r.stroke, e.edgeStroke, a.muted) ?? "#64748b"],
		["--kg-text", q(s.text, i.color, e.foreground, a.foreground) ?? "#111827"],
		["--kg-text-muted", q(s.textMuted) ?? "#64748b"],
		["--kg-accent", q(s.accent, e.accent, a.accent) ?? "#2563eb"],
		["--kg-font-family", q(l.family, e.fontFamily, i.fontFamily) ?? "system-ui, sans-serif"]
	];
	for (let e of Object.keys(s).sort()) {
		let t = J(s[e]);
		t && u.push([`--kg-color-${La(e)}`, t]);
	}
	for (let e of Object.keys(c).sort()) {
		let t = c[e];
		typeof t == "number" && Number.isFinite(t) && u.push([`--kg-radius-${La(e)}`, `${t}px`]);
	}
	return u.map(([e, t]) => `${e}:${t}`).join(";");
}
var ba = Va(".kg-scene{color:var(--kg-text)}.kg-node-shape{vector-effect:non-scaling-stroke}.kg-nodes>.kg-node .kg-node-shape:not([fill]){fill:var(--kg-node-fill)}.kg-node text{stroke:none}.kg-node-label,.kg-node-body,.kg-node-icon,.kg-edge-label{font-family:var(--kg-font-family)}.kg-node-label{fill:var(--kg-text);font-size:13px;font-weight:600}.kg-node-body{fill:var(--kg-text-muted);font-size:11px;font-weight:400}.kg-node-icon-bg{fill:var(--kg-accent);stroke:none}.kg-node-icon{fill:white;font-size:10px;font-weight:700}.kg-node-motif{fill:none;stroke:var(--kg-accent);stroke-width:1.5;stroke-linecap:round;stroke-linejoin:round}.kg-node-motif .kg-motif-backed{fill:var(--kg-background)}.kg-node-motif .kg-motif-solid{fill:var(--kg-accent)}.kg-edge{vector-effect:non-scaling-stroke}.kg-edge-label{fill:var(--kg-text)}.kg-node--interactive{cursor:pointer;outline:none}.kg-node--interactive:focus-visible>.kg-node-shape,.kg-node--interactive[data-inspected=true]>.kg-node-shape{stroke:var(--kg-accent);stroke-width:2}.kg-node--interactive:hover>.kg-node-shape{filter:brightness(1.06)}@keyframes kg-flow{to{stroke-dashoffset:-1000}}.kg-edge--flowing{animation:kg-flow 40s linear infinite}.kg-scene[data-paused] .kg-edge--flowing,.kg-scene[data-reduced-motion] .kg-edge--flowing{animation-play-state:paused}@media(prefers-reduced-motion:reduce){.kg-edge--flowing{animation:none}}");
function G(e, t, n) {
	let r = /* @__PURE__ */ new Set(), i = t.filter((e) => typeof e[1] == "string").filter(([e]) => !r.has(e) && (r.add(e), !0)).map(([e, t]) => ` ${e}="${Ha(t)}"`).join("");
	return n ? `<${e}${i}>${n}</${e}>` : `<${e}${i}/>`;
}
function xa(e) {
	return Object.keys(e).sort().flatMap((t) => {
		let n = e[t];
		if (typeof n != "string" && typeof n != "number" && typeof n != "boolean") return [];
		let r = t.toLowerCase().replace(/[^a-z0-9_.:-]+/g, "-").replace(/^-+|-+$/g, "");
		return r ? [[`data-${r}`, String(n)]] : [];
	});
}
function Sa(e, t) {
	return J(e.id) ?? `node-${t + 1}`;
}
function Ca(e) {
	let t = q(e.kind, e.type, K(e.shape).type) ?? "rect";
	return t === "shape" ? (q(e.shape) ?? "rectangle").toLowerCase() : t.toLowerCase();
}
function wa(e, t = 80, n = 40) {
	let r = K(e.bounds), i = K(e.size);
	return {
		x: X(e.x, X(r.x, 0)),
		y: X(e.y, X(r.y, 0)),
		width: Math.max(0, X(e.width, X(r.width, X(i.width, t)))),
		height: Math.max(0, X(e.height, X(r.height, X(i.height, n))))
	};
}
function Ta(e) {
	return ka(e.interactive) === !0 || ka(K(e.interaction).enabled) === !0 || e.action !== void 0 || e.onActivate !== void 0 || e.href !== void 0;
}
function Ea(e) {
	return Array.isArray(e) ? e.filter((e) => Da(e)) : [];
}
function K(e) {
	return Da(e) ? e : {};
}
function Da(e) {
	return typeof e == "object" && !!e && !Array.isArray(e);
}
function Oa(...e) {
	let t = {};
	for (let n of e) for (let [e, r] of Object.entries(n)) t[e] = r;
	return t;
}
function q(...e) {
	for (let t of e) if (typeof t == "string" && t.length > 0) return t;
}
function J(e) {
	return typeof e == "string" && e.length > 0 ? e : void 0;
}
function ka(e) {
	return typeof e == "boolean" ? e : void 0;
}
function Y(...e) {
	for (let t of e) if (typeof t == "number" && Number.isFinite(t)) return t;
}
function X(e, t) {
	return typeof e == "number" && Number.isFinite(e) ? e : t;
}
function Aa(e, t) {
	let n = X(e, t);
	return n > 0 ? n : t;
}
function ja(e, t) {
	return Math.max(0, Math.min(1, e ?? t));
}
function Ma(e, t, n, r) {
	let i = X(e, r);
	return Math.max(t, Math.min(n, Math.trunc(i)));
}
function Z(e, t) {
	let n = Number(e.toFixed(t));
	return Object.is(n, -0) ? "0" : String(n);
}
function Na(e, t) {
	return typeof e == "number" && Number.isFinite(e) ? Z(e, t) : void 0;
}
var Pa = /* @__PURE__ */ new Set([
	"canvas",
	"surface",
	"surfaceRaised",
	"surfaceMuted",
	"text",
	"textMuted",
	"accent",
	"accentContrast",
	"info",
	"success",
	"warning",
	"danger",
	"connector",
	"border"
]);
function Fa(e) {
	if (e) return Pa.has(e) ? `var(--kg-color-${La(e)})` : e;
}
function Ia(e, t, n) {
	return typeof e == "number" && Number.isFinite(e) ? Z(e, t) : typeof e == "string" && e.length > 0 ? `var(--kg-${n}-${La(e)})` : void 0;
}
function La(e) {
	return e.replace(/([a-z0-9])([A-Z])/g, "$1-$2").toLowerCase().replace(/[^a-z0-9-]+/g, "-");
}
function Ra(e, t) {
	return typeof e == "string" ? e : Array.isArray(e) ? e.flatMap((e) => Array.isArray(e) && e.length >= 2 ? [`${Z(X(e[0], 0), t)},${Z(X(e[1], 0), t)}`] : Da(e) ? [`${Z(X(e.x, 0), t)},${Z(X(e.y, 0), t)}`] : []).join(" ") : "";
}
function za(...e) {
	return e.filter((e) => typeof e == "string" && e.length > 0).join(" ");
}
function Ba(e) {
	let t = e.trim().replace(/[^A-Za-z0-9_.:-]+/g, "-").replace(/^-+|-+$/g, "");
	return (/^[A-Za-z_]/.test(t) ? t : `id-${t || "scene"}`) || "kineglyph-scene";
}
function Va(e) {
	return e.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}
function Ha(e) {
	return Va(e).replace(/"/g, "&quot;").replace(/'/g, "&apos;");
}
//#endregion
//#region ../anime/dist/index.js
var Ua = class {
	#e;
	#t;
	#n;
	#r;
	#i;
	#a;
	#o;
	#s = !1;
	#c = !1;
	#l;
	constructor(e) {
		this.#e = e.root, this.#t = e.scene, this.#o = e.reducedMotion ?? !1, this.#r = e.onFrame, this.#i = e.onPlaybackChange, this.#l = e.scene.timeline?.duration ?? 0, this.#n = Oe({ root: e.root }), this.#n.add(() => {
			let e = Ce({
				autoplay: !1,
				duration: Math.max(1, this.#l),
				onUpdate: (e) => {
					this.#c || this.#f(this.#s ? this.#l : e.currentTime);
				},
				onComplete: () => {
					this.#c || (this.#s = !0, this.#f(this.#l), this.#d(!0), this.#i?.(!1));
				},
				onPause: () => {
					this.#c || (this.#d(!0), this.#i?.(!1));
				}
			});
			return this.#a = e, () => e.cancel();
		}), this.#d(!0), this.#f(this.#o ? this.#l : 0);
	}
	get duration() {
		return this.#l;
	}
	get scene() {
		return this.#t;
	}
	get time() {
		return this.#o ? this.#l : this.#a?.currentTime ?? 0;
	}
	get playing() {
		return this.#a !== void 0 && !this.#a.paused && !this.#a.completed;
	}
	setScene(e, t = {}) {
		this.#_();
		let n = this.#u();
		this.#t = e;
		let r = e.timeline?.duration ?? 0, i = Wa(t.time ?? this.time, 0, r);
		r !== this.#l && (this.#l = r, this.#a?.cancel(), this.#n.add(() => {
			let e = Ce({
				autoplay: !1,
				duration: Math.max(1, r),
				onUpdate: (e) => {
					this.#c || this.#f(this.#s ? this.#l : e.currentTime);
				},
				onComplete: () => {
					this.#c || (this.#s = !0, this.#f(this.#l), this.#d(!0), this.#i?.(!1));
				},
				onPause: () => {
					this.#c || (this.#d(!0), this.#i?.(!1));
				}
			});
			return this.#a = e, () => e.cancel();
		})), this.#s = i >= this.#l, this.#a?.seek(i, !0), this.#f(this.#o ? this.#l : i), n && !this.#s && !this.#o && this.play();
	}
	play() {
		if (this.#_(), this.#o || this.#l === 0) {
			this.#f(this.#l);
			return;
		}
		this.#a?.completed || this.#s ? (this.#s = !1, this.#a?.restart()) : this.#a?.play(), this.#d(!1), this.#i?.(!0);
	}
	pause() {
		this.#_(), this.#a?.pause(), this.#d(!0), this.#i?.(!1);
	}
	restart(e = !0) {
		if (this.#_(), this.#o) {
			this.#f(this.#l);
			return;
		}
		this.#s = !1, this.#a?.pause().seek(0, !0), this.#f(0), e && this.play();
	}
	seek(e) {
		this.#_();
		let t = this.#o ? this.#l : Wa(e, 0, this.#l);
		this.#s = t >= this.#l, this.#s && this.#a?.pause(), this.#a?.seek(t, !0), this.#f(t);
	}
	setReducedMotion(e) {
		this.#_(), this.#o = e, e ? (this.#s = !0, this.#a?.pause(), this.#e.setAttribute("data-reduced-motion", "true"), this.#i?.(!1)) : (this.#s = !1, this.#e.removeAttribute("data-reduced-motion")), this.#f(e ? this.#l : this.time);
	}
	applyFrame(e) {
		return this.#_(), this.#f(Wa(e, 0, this.#l));
	}
	dispose() {
		this.#c || (this.#c = !0, this.#a?.cancel(), this.#n.revert(), this.#a = void 0);
	}
	#u() {
		return this.playing;
	}
	#d(e) {
		let t = qa(this.#e, "svg") ? this.#e : this.#e.querySelector("svg");
		t !== null && (e ? t.setAttribute("data-paused", "true") : t.removeAttribute("data-paused"));
	}
	#f(e) {
		let t = Fi(this.#t, e);
		if (this.#c) return t;
		let n = this.#t.theme.accent;
		for (let e of t.nodes) this.#p(e, n);
		for (let e of t.edges) this.#m(e, n);
		return this.#e.style.setProperty("--kg-time", String(t.time)), this.#e.style.setProperty("--kg-timeline-progress", String(t.progress)), this.#r?.(t), t;
	}
	#p(e, t) {
		for (let n of this.#g(`[data-node-id="${Ja(e.id)}"]`)) {
			if (!(n instanceof SVGElement)) continue;
			n.style.opacity = String(e.state.opacity), n.style.transformBox = "view-box", n.style.transformOrigin = "0 0";
			let r = aa(e, e.state.translateX, e.state.translateY, e.state.scale), i = [r.tx !== 0 || r.ty !== 0 ? `translate(${Ga(r.tx)}px, ${Ga(r.ty)}px)` : "", r.scale === 1 ? "" : `scale(${Ga(r.scale)})`].filter(Boolean).join(" ");
			n.style.transform = i.length > 0 ? i : "none";
			let a = e.state.revealX ?? 1, o = e.state.revealY ?? 1, s = n.querySelector(`[data-reveal-clip="${Ja(e.id)}"]`);
			if (s instanceof SVGElement) {
				let t = sa(e, a, o, e.revealAnchor);
				s.setAttribute("x", String(Ga(t.x))), s.setAttribute("y", String(Ga(t.y))), s.setAttribute("width", String(Ga(Math.max(0, t.width)))), s.setAttribute("height", String(Ga(Math.max(0, t.height)))), n.setAttribute("data-reveal-x", String(Ga(a))), n.setAttribute("data-reveal-y", String(Ga(o)));
			}
			n.style.setProperty("--kg-progress", String(e.state.progress));
			let c = e.state.highlight ?? 0;
			n.style.setProperty("--kg-highlight", String(c)), c > 0 ? n.setAttribute("data-highlight", String(Ga(c))) : n.removeAttribute("data-highlight");
			let l = n.querySelector(`.kg-node-shape[data-shape-of="${Ja(e.id)}"]`);
			if (l instanceof SVGElement) {
				let n = qa(l, "path"), r = n ? Ka(e) : 1, i = e.appearance.stroke === "none" ? t : e.appearance.stroke, a = e.appearance.stroke === "none" ? 0 : e.appearance.strokeWidth, o = a;
				if (e.appearance.stroke !== "none" || c > 0) {
					let e = Qi(i, t, c, a);
					l.setAttribute("stroke", e.stroke), o = e.strokeWidth, o > 0 && l.setAttribute("stroke-width", String(Ga(o / r)));
				}
				if (n && l.hasAttribute("data-path-length")) {
					let t = Number(l.getAttribute("data-path-length")) || 0, n = ta(l.getAttribute("data-dash") ?? "solid", o / r, t, e.state.progress);
					n.pathLength === void 0 ? l.removeAttribute("pathLength") : l.setAttribute("pathLength", n.pathLength), n.dasharray === void 0 ? l.removeAttribute("stroke-dasharray") : l.setAttribute("stroke-dasharray", n.dasharray);
				}
			}
			let u = n.querySelector(`text[data-text-of="${Ja(e.id)}"]`);
			if (u instanceof SVGElement) {
				let t = [...u.querySelectorAll("tspan")], n = e.state.progress >= 1 ? t.length : Math.max(0, Math.round(t.length * e.state.progress));
				t.forEach((e, t) => {
					t < n ? e.removeAttribute("opacity") : e.setAttribute("opacity", "0");
				});
			}
		}
	}
	#m(e, t) {
		let n = this.#g(`[data-edge-id="${Ja(e.id)}"]`);
		for (let r of n) {
			if (!qa(r, "path")) continue;
			r.style.opacity = String(e.state.opacity);
			let n = e.state.highlight ?? 0, i = Qi(e.appearance.stroke, t, n, e.appearance.strokeWidth);
			r.setAttribute("stroke", i.stroke), r.setAttribute("stroke-width", String(Ga(i.strokeWidth)));
			let a = r.getAttribute("data-dash") ?? e.dash ?? "solid", o = Number(r.getAttribute("data-length")) || e.length || 0, s = ea(a, i.strokeWidth, o, e.state.progress);
			s.pathLength === void 0 ? r.removeAttribute("pathLength") : r.setAttribute("pathLength", s.pathLength), s.dasharray === void 0 ? r.removeAttribute("stroke-dasharray") : r.setAttribute("stroke-dasharray", s.dasharray), r.classList.toggle("kg-edge--flowing", a === "flow" && e.state.progress >= 1), r.style.strokeDasharray = "", r.style.strokeDashoffset = "";
			let c = r.getAttribute("data-head") ?? e.head ?? "none", l = r.getAttribute("data-tail") ?? e.tail ?? "none";
			this.#h(r, "marker-end", c, i.stroke, e.state.progress >= 1), this.#h(r, "marker-start", l, i.stroke, e.state.progress > 0), n > 0 ? r.setAttribute("data-highlight", String(Ga(n))) : r.removeAttribute("data-highlight");
		}
		let r = e.state.flow ?? 0, i = e.packets ?? [];
		for (let t of this.#g(`[data-edge-packet="${Ja(e.id)}"]`)) {
			if (!qa(t, "circle")) continue;
			let n = i[Number(t.getAttribute("data-packet-index") ?? "0")];
			n !== void 0 && (t.setAttribute("cx", String(Ga(n.x))), t.setAttribute("cy", String(Ga(n.y)))), t.setAttribute("opacity", String(Ga(r * e.state.opacity)));
		}
		for (let t of this.#g(`[data-edge-group="${Ja(e.id)}"] .kg-edge-label`)) t instanceof SVGElement && (t.style.opacity = String(e.state.opacity));
	}
	#h(e, t, n, r, i) {
		if (n === "none" || !i) {
			e.removeAttribute(t);
			return;
		}
		let a = e.ownerSVGElement;
		if (a === null) return;
		let o = a.id, s = Bi(o, n, r);
		if (a.querySelector(`#${Ja(s)}`) === null) {
			let e = a.querySelector(":scope > defs");
			e === null && (e = a.ownerDocument.createElementNS("http://www.w3.org/2000/svg", "defs"), a.insertBefore(e, a.firstChild)), e.insertAdjacentHTML("beforeend", Hi(o, n, r));
		}
		e.setAttribute(t, `url(#${s})`);
	}
	#g(e) {
		return Array.from(this.#e.querySelectorAll(e));
	}
	#_() {
		if (this.#c) throw Error("KineglyphSceneAnimator has been disposed");
	}
};
function Wa(e, t, n) {
	return Number.isFinite(e) ? Math.min(n, Math.max(t, e)) : t;
}
function Ga(e) {
	return Math.round(e * 1e4) / 1e4;
}
function Ka(e) {
	let t = e.path, n = t?.viewBox?.width, r = t?.viewBox?.height, i = typeof n == "number" && n > 0 ? n : 24, a = typeof r == "number" && r > 0 ? r : 24;
	return Math.max(1e-6, Math.min(e.width / i, e.height / a));
}
function qa(e, t) {
	return e instanceof SVGElement && e.tagName.toLowerCase() === t;
}
function Ja(e) {
	return typeof CSS < "u" && typeof CSS.escape == "function" ? CSS.escape(e) : e.replace(/["\\]/g, "\\$&");
}
//#endregion
//#region src/shaders.ts
var Ya = "http://www.w3.org/2000/svg", Xa = "http://www.w3.org/1999/xhtml", Za = class {
	#e;
	constructor(e) {
		this.#e = e;
	}
	seek(e) {
		for (let t of this.#e) t.seek(e);
	}
	dispose() {
		for (let e of this.#e) e.dispose();
	}
};
function Qa(e, t) {
	let n = [], r = e.querySelectorAll("[data-shader]");
	for (let e of r) {
		let r = eo(e.dataset.shader?.split(/\s+/)[0]);
		if (r === void 0 || e.tagName.toLowerCase() !== "rect") continue;
		let i = $a(e, r);
		i !== void 0 && (i.seek(t), n.push(i));
	}
	return n.length === 0 ? void 0 : new Za(n);
}
function $a(e, t) {
	let n = io(e, "width"), r = io(e, "height");
	if (n === void 0 || r === void 0) return;
	let i = e.ownerDocument, a = i.createElementNS(Ya, "foreignObject");
	for (let t of [
		"x",
		"y",
		"width",
		"height"
	]) {
		let n = e.getAttribute(t);
		n !== null && a.setAttribute(t, n);
	}
	a.setAttribute("aria-hidden", "true"), a.setAttribute("focusable", "false"), a.setAttribute("data-kineglyph-shader-surface", t), a.style.pointerEvents = "none", a.style.overflow = "hidden";
	let o = Math.max(0, ro(e, "rx") ?? 0);
	o > 0 && (a.style.borderRadius = `${o}px`);
	let s = i.createElementNS(Xa, "canvas"), c = Math.min(2, Math.max(1, e.ownerDocument.defaultView?.devicePixelRatio ?? 1));
	s.width = Math.max(1, Math.round(n * c)), s.height = Math.max(1, Math.round(r * c)), s.style.width = "100%", s.style.height = "100%", s.style.display = "block", s.style.pointerEvents = "none", s.setAttribute("aria-hidden", "true"), a.append(s);
	let l;
	try {
		l = s.getContext("webgl", {
			alpha: !0,
			antialias: !1,
			depth: !1,
			premultipliedAlpha: !0,
			preserveDrawingBuffer: !1
		});
	} catch {
		return;
	}
	if (l === null) return;
	let u = ao(l, so, co);
	if (u === void 0) return;
	let d = l.createBuffer();
	if (d === null) {
		l.deleteProgram(u);
		return;
	}
	l.bindBuffer(l.ARRAY_BUFFER, d), l.bufferData(l.ARRAY_BUFFER, new Float32Array([
		-1,
		-1,
		1,
		-1,
		-1,
		1,
		1,
		1
	]), l.STATIC_DRAW), l.useProgram(u);
	let f = l.getAttribLocation(u, "a_position");
	l.enableVertexAttribArray(f), l.vertexAttribPointer(f, 2, l.FLOAT, !1, 0, 0), l.viewport(0, 0, s.width, s.height), l.enable(l.BLEND), l.blendFunc(l.ONE, l.ONE_MINUS_SRC_ALPHA);
	let p = l.getUniformLocation(u, "u_time"), m = l.getUniformLocation(u, "u_mode"), h = l.getUniformLocation(u, "u_strength");
	return l.uniform1i(m, to(t)), l.uniform1f(h, no(e, t)), e.before(a), {
		foreign: a,
		seek(e) {
			l?.useProgram(u), l?.uniform1f(p, Math.max(0, e) / 1e3), l?.drawArrays(l.TRIANGLE_STRIP, 0, 4);
		},
		dispose() {
			a.remove(), l?.deleteBuffer(d), l?.deleteProgram(u), l = null;
		}
	};
}
function eo(e) {
	return e === "frosted-glass" || e === "iridescence" || e === "liquid" || e === "grain" ? e : void 0;
}
function to(e) {
	return e === "frosted-glass" ? 0 : e === "iridescence" ? 1 : e === "liquid" ? 2 : 3;
}
function no(e, t) {
	let n = {};
	try {
		let t = e.getAttribute("data-shader-uniforms"), r = t === null ? {} : JSON.parse(t);
		typeof r == "object" && r && !Array.isArray(r) && (n = r);
	} catch {
		n = {};
	}
	let r = t === "frosted-glass" ? [
		"refraction",
		"grain",
		"strength"
	] : t === "grain" ? ["grain", "strength"] : ["strength", "intensity"];
	for (let e of r) {
		let t = n[e];
		if (typeof t == "number" && Number.isFinite(t)) return Math.max(0, t);
	}
	return t === "iridescence" ? .16 : t === "liquid" ? 2.5 : .08;
}
function ro(e, t) {
	let n = Number(e.getAttribute(t));
	return Number.isFinite(n) ? n : void 0;
}
function io(e, t) {
	let n = ro(e, t);
	return n !== void 0 && n > 0 ? n : void 0;
}
function ao(e, t, n) {
	let r = oo(e, e.VERTEX_SHADER, t), i = oo(e, e.FRAGMENT_SHADER, n);
	if (r === void 0 || i === void 0) {
		r !== void 0 && e.deleteShader(r), i !== void 0 && e.deleteShader(i);
		return;
	}
	let a = e.createProgram();
	if (a !== null) {
		if (e.attachShader(a, r), e.attachShader(a, i), e.linkProgram(a), e.deleteShader(r), e.deleteShader(i), !e.getProgramParameter(a, e.LINK_STATUS)) {
			e.deleteProgram(a);
			return;
		}
		return a;
	}
}
function oo(e, t, n) {
	let r = e.createShader(t);
	if (r !== null) {
		if (e.shaderSource(r, n), e.compileShader(r), !e.getShaderParameter(r, e.COMPILE_STATUS)) {
			e.deleteShader(r);
			return;
		}
		return r;
	}
}
var so = "\nattribute vec2 a_position;\nvarying vec2 v_uv;\nvoid main() {\n  v_uv = a_position * 0.5 + 0.5;\n  gl_Position = vec4(a_position, 0.0, 1.0);\n}", co = "\nprecision highp float;\nvarying vec2 v_uv;\nuniform float u_time;\nuniform float u_strength;\nuniform int u_mode;\n\nfloat hash(vec2 p) {\n  return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453123);\n}\n\nvoid main() {\n  vec2 uv = v_uv;\n  float grain = hash(floor(uv * 420.0) + floor(u_time * 12.0));\n  vec3 color;\n  float alpha;\n  if (u_mode == 0) {\n    float caustic = sin((uv.x * 5.0 + uv.y * 3.0 + u_time * 0.22) * 6.28318) * 0.5 + 0.5;\n    color = mix(vec3(0.64, 0.82, 1.0), vec3(0.92, 0.72, 1.0), caustic);\n    alpha = 0.035 + grain * min(0.11, u_strength * 0.7);\n  } else if (u_mode == 1) {\n    vec3 phase = vec3(0.0, 0.33, 0.67);\n    color = 0.5 + 0.5 * cos(6.28318 * (phase + uv.x * 0.8 + uv.y * 0.35 + u_time * 0.04));\n    alpha = min(0.24, 0.055 + u_strength * 0.55);\n  } else if (u_mode == 2) {\n    float wave = sin(uv.x * 18.0 + sin(uv.y * 11.0 + u_time * 0.4) * 1.8 + u_time * 0.3);\n    color = mix(vec3(0.18, 0.64, 0.95), vec3(0.66, 0.34, 0.94), wave * 0.5 + 0.5);\n    alpha = min(0.22, 0.045 + u_strength * 0.025);\n  } else {\n    color = vec3(grain);\n    alpha = min(0.18, 0.025 + u_strength);\n  }\n  gl_FragColor = vec4(color * alpha, alpha);\n}", lo = "kineglyph-web-styles", uo = "\n.kg-figure{box-sizing:border-box;position:relative;overflow:hidden;border:1px solid var(--kg-shell-border);border-radius:var(--kg-shell-radius);background:var(--kg-shell-background);color:var(--kg-shell-text);font-family:var(--kg-shell-font)}\n.kg-figure *{box-sizing:border-box}\n.kg-figure__stage{position:relative;width:100%;min-height:120px;background:var(--kg-shell-background);overflow:hidden}\n.kg-figure__stage svg{display:block;width:100%;height:auto;overflow:visible}\n.kg-figure__stage [data-node-id]{transition:filter 160ms ease}\n.kg-figure__stage [data-node-id][data-inspected=true]>.kg-node-shape,.kg-figure__stage [data-node-id][data-selected=true]>.kg-node-shape{stroke:var(--kg-shell-accent);stroke-width:2}\n.kg-figure__stage .kg-node--interactive:hover>.kg-node-shape{stroke:var(--kg-shell-accent)}\n.kg-figure__stage .kg-node--interactive:focus-visible>.kg-node-shape{stroke:var(--kg-shell-accent);stroke-width:2.5}\n.kg-figure__stage .kg-edge-group[role=img]:hover .kg-edge{filter:brightness(1.25)}\n.kg-figure__readout{display:grid;grid-template-columns:minmax(110px,.4fr) minmax(140px,.7fr) minmax(220px,1.5fr);gap:16px;align-items:baseline;min-height:64px;padding:16px 22px;border-top:1px solid var(--kg-shell-border);background:var(--kg-shell-surface)}\n.kg-figure__readout strong{font-size:15px}.kg-figure__body{color:var(--kg-shell-muted);font-size:13px;line-height:1.45}\n.kg-figure__fields{display:grid;grid-template-columns:auto 1fr;gap:2px 12px;margin:8px 0 0;font-size:12px}\n.kg-figure__fields dt{color:var(--kg-shell-muted);font-weight:600}\n.kg-figure__fields dd{margin:0;color:var(--kg-shell-text);font-variant-numeric:tabular-nums}\n.kg-figure__eyebrow{text-transform:uppercase;letter-spacing:.13em;color:var(--kg-shell-accent);font-size:10px;font-weight:700}\n.kg-figure__machine{display:flex;flex-wrap:wrap;align-items:center;gap:8px;padding:12px 16px;border-top:1px solid var(--kg-shell-border);background:var(--kg-shell-surface)}\n.kg-figure__machine-group{display:flex;flex-wrap:wrap;align-items:center;gap:6px;margin-right:12px}\n.kg-figure__machine-label{color:var(--kg-shell-muted);font-size:10px;font-weight:700;letter-spacing:.1em;text-transform:uppercase;margin-right:4px}\n.kg-figure__controls{display:flex;align-items:center;gap:10px;padding:12px 16px;border-top:1px solid var(--kg-shell-border);background:var(--kg-shell-surface)}\n.kg-figure__machine+.kg-figure__controls{border-top:none;padding-top:8px}\n.kg-figure button{appearance:none;border:1px solid var(--kg-shell-border);border-radius:4px;padding:8px 12px;background:var(--kg-shell-background);color:var(--kg-shell-text);font:600 12px/1 var(--kg-shell-font);cursor:pointer}\n.kg-figure button:hover:not(:disabled),.kg-figure button:focus-visible{border-color:var(--kg-shell-accent);outline:none}\n.kg-figure button:focus-visible{box-shadow:0 0 0 2px color-mix(in srgb,var(--kg-shell-accent),transparent 70%)}\n.kg-figure button:disabled{opacity:.42;cursor:not-allowed}\n.kg-figure button[aria-pressed=true]{border-color:var(--kg-shell-accent);background:color-mix(in srgb,var(--kg-shell-accent),var(--kg-shell-background) 84%);color:var(--kg-shell-text)}\n.kg-figure__scrubber{display:flex;align-items:center;gap:10px;flex:1;min-width:160px;color:var(--kg-shell-muted);font-size:11px;text-transform:uppercase;letter-spacing:.08em}\n.kg-figure__scrubber input{width:100%;accent-color:var(--kg-shell-accent)}\n.kg-figure__controls output{min-width:48px;text-align:right;color:var(--kg-shell-muted);font-variant-numeric:tabular-nums;font-size:12px}\n.kg-figure--compact .kg-figure__readout{grid-template-columns:1fr;gap:5px;min-height:96px}\n.kg-figure--compact .kg-figure__controls{flex-wrap:wrap}\n.kg-figure--compact .kg-figure__scrubber{order:3;flex-basis:100%}\n.kg-live-surface{position:absolute;z-index:3;overflow:hidden;transform-origin:center;opacity:0;background:transparent}\n.kg-live-surface>canvas,.kg-live-surface>iframe,.kg-live-surface>model-viewer{display:block;width:100%;height:100%;border:0}\n.kg-figure__live{position:absolute;width:1px;height:1px;overflow:hidden;clip:rect(0 0 0 0);white-space:nowrap}\n@media(prefers-reduced-motion:reduce){.kg-figure__stage [data-node-id]{transition:none}}\n";
function fo(e) {
	let t = e.ownerDocument;
	if (t.getElementById("kineglyph-web-styles") !== null) return;
	let n = t.createElement("style");
	n.id = lo, n.textContent = uo, (t.head ?? t.documentElement).append(n);
}
//#endregion
//#region src/surfaces.ts
var po = class {
	#e;
	#t;
	#n = [];
	#r = !1;
	constructor(e, t, n) {
		this.#e = t, this.#t = n;
		for (let r of t.nodes) {
			if (r.kind !== "image" || r.image?.live !== !0) continue;
			let i = n.renderers?.[r.id];
			if (i === void 0) continue;
			let a = e.ownerDocument.createElement("div");
			a.className = "kg-live-surface", a.dataset.surfaceId = r.id, a.setAttribute("aria-label", r.image.alt), ho(a, r, t), e.append(a);
			let o = _o(e, r.id), s = {
				nodeId: r.id,
				layer: a,
				fallback: o,
				abort: new AbortController(),
				handle: void 0
			};
			this.#n.push(s), this.#i(s, r, i);
		}
	}
	async #i(e, t, n) {
		try {
			let r = mo(await n({
				element: e.layer,
				node: t,
				scene: this.#e,
				theme: this.#t.theme,
				machineState: this.#t.machineState,
				signals: this.#t.signals,
				time: this.#t.time,
				signal: e.abort.signal,
				send: this.#t.send
			}));
			if (e.abort.signal.aborted || this.#r) {
				r?.destroy?.();
				return;
			}
			if (r?.mounted === !1) {
				e.layer.remove();
				return;
			}
			if (e.handle = r, await r?.ready, e.abort.signal.aborted || this.#r) return;
			e.layer.dataset.ready = "true", go(e.layer, t), e.fallback !== void 0 && (e.fallback.style.opacity = "0");
		} catch (t) {
			e.abort.signal.aborted || this.#t.onError?.(e.nodeId, t), e.layer.remove();
		}
	}
	update(e) {
		for (let t of this.#n) {
			let n = e.nodes.find((e) => e.id === t.nodeId);
			n !== void 0 && (t.layer.dataset.ready === "true" && go(t.layer, n), t.handle?.update?.({
				frame: e,
				node: n,
				machineState: e.machineState,
				signals: e.signals ?? {},
				time: e.time
			}));
		}
	}
	dispose() {
		if (!this.#r) {
			this.#r = !0;
			for (let e of this.#n) e.abort.abort(), e.handle?.destroy?.(), e.layer.remove(), e.fallback !== void 0 && e.fallback.style.removeProperty("opacity");
			this.#n.length = 0;
		}
	}
};
function mo(e) {
	if (e !== void 0) return typeof e == "function" ? { destroy: e } : e;
}
function ho(e, t, n) {
	e.style.left = `${t.x / n.width * 100}%`, e.style.top = `${t.y / n.height * 100}%`, e.style.width = `${t.width / n.width * 100}%`, e.style.height = `${t.height / n.height * 100}%`, e.style.borderRadius = `${t.appearance.radius}px`, e.style.opacity = "0";
}
function go(e, t) {
	e.style.opacity = String(t.state.opacity), e.style.transform = `translate(${t.state.translateX}px, ${t.state.translateY}px) scale(${t.state.scale})`;
}
function _o(e, t) {
	for (let n of e.querySelectorAll("image[data-live=true]")) if (n.closest("[data-node-id]")?.getAttribute("data-node-id") === t) return n;
}
function vo(e) {
	return async (t) => {
		if (t.element.ownerDocument.defaultView?.customElements.get("model-viewer") === void 0) return { mounted: !1 };
		let n = t.element.ownerDocument.createElement("model-viewer");
		n.setAttribute("alt", e.alt ?? t.node.image?.alt ?? t.node.label), n.setAttribute("loading", "eager"), n.setAttribute("interaction-prompt", "none"), n.setAttribute("tone-mapping", "neutral"), e.cameraControls !== !1 && n.setAttribute("camera-controls", ""), e.autoRotate === !0 && n.setAttribute("auto-rotate", "");
		for (let [t, r] of Object.entries(e.attributes ?? {})) n.setAttribute(t, r);
		n.style.display = "block", n.style.width = "100%", n.style.height = "100%", n.style.background = "transparent", t.element.append(n);
		let r = typeof e.source == "function" ? await e.source(t) : e.source;
		if (t.signal.aborted) return { mounted: !1 };
		let i = yo(r), a = new Promise((e, r) => {
			n.addEventListener("load", () => e(), { once: !0 }), n.addEventListener("error", () => r(/* @__PURE__ */ Error(`model-viewer could not load ${t.node.id}`)), { once: !0 }), t.signal.addEventListener("abort", () => r(new DOMException("Aborted", "AbortError")), { once: !0 });
		});
		return n.setAttribute("src", i.url), {
			ready: a,
			destroy() {
				n.remove(), i.revoke?.();
			}
		};
	};
}
function yo(e) {
	if (typeof e == "string") return { url: e };
	let t = globalThis.URL, n = globalThis.Blob, r = e instanceof Uint8Array ? new Uint8Array(e).buffer : e, i = e instanceof n ? e : new n([r], { type: "model/gltf-binary" }), a = t.createObjectURL(i);
	return {
		url: a,
		revoke: () => t.revokeObjectURL(a)
	};
}
//#endregion
//#region src/index.ts
var bo = 0;
function xo(e, t) {
	return new Co(e, t);
}
var So = class {
	#e = /* @__PURE__ */ new Map();
	on(e, t) {
		let n = this.#e.get(e) ?? /* @__PURE__ */ new Set();
		return n.add(t), this.#e.set(e, n), () => {
			n.delete(t);
		};
	}
	emit(e, t) {
		for (let n of this.#e.get(e) ?? []) n(t);
	}
	clear() {
		this.#e.clear();
	}
}, Co = class {
	element;
	stage;
	id;
	machine;
	#e;
	#t;
	#n;
	#r;
	#i;
	#a;
	#o;
	#s;
	#c;
	#l;
	#u = !1;
	#d = 0;
	#f = !1;
	#p = new So();
	#m = [];
	#h;
	#g;
	#_;
	#v;
	#y;
	#b;
	#x;
	#S;
	#C;
	#w;
	constructor(e, t) {
		this.element = e, this.#n = t, this.#e = t.scene, this.#t = t.theme ?? Mt, bo += 1, this.id = t.idPrefix ?? `kineglyph-${bo.toString(36)}`, fo(e);
		let n = e.ownerDocument;
		e.replaceChildren(), e.classList.add("kg-figure-host"), this.#h = n.createElement("section"), this.#h.className = ["kg-figure", t.className].filter(Boolean).join(" "), this.stage = n.createElement("div"), this.stage.className = "kg-figure__stage", this.#h.append(this.stage), this.#C = n.createElement("div"), this.#C.className = "kg-figure__live", this.#C.setAttribute("aria-live", "polite"), this.#h.append(this.#C), t.readout !== !1 && (this.#g = n.createElement("div"), this.#g.className = "kg-figure__readout", this.#g.innerHTML = "<span class=\"kg-figure__eyebrow\"></span><strong></strong><div class=\"kg-figure__body\"></div>", this.#h.append(this.#g)), t.machineControls !== !1 && (this.#_ = n.createElement("div"), this.#_.className = "kg-figure__machine", this.#_.hidden = !0, this.#h.append(this.#_)), t.controls !== !1 && (this.#v = n.createElement("div"), this.#v.className = "kg-figure__controls", this.#h.append(this.#v), this.#j(n)), e.append(this.#h), this.#c = t.reducedMotion ?? Mo(e), this.#s = Math.max(280, Math.round(t.width ?? jo(e))), Ao(this.#e) && this.#e.machine !== void 0 && (this.machine = new Ln(this.#e.machine, {
			...t.initialState === void 0 ? {} : { initialState: t.initialState },
			history: t.history ?? !1
		})), this.#r = this.#D(), this.#O(!0), this.#L(n), this.#R(e), t.width === void 0 && this.#z(e), e.setAttribute("aria-busy", "false");
	}
	get scene() {
		return this.#r;
	}
	get state() {
		return {
			time: this.#c ? this.#T : this.#d,
			duration: this.#T,
			playing: this.#f,
			reducedMotion: this.#c,
			width: this.#s,
			layout: this.#r.layoutName,
			machineState: this.machine?.state,
			inspected: this.#l,
			destroyed: this.#u
		};
	}
	play() {
		this.#E(), this.#i?.play();
	}
	pause() {
		this.#E(), this.#i?.pause();
	}
	restart(e = !0) {
		this.#E(), this.#i?.restart(e);
	}
	seek(e) {
		this.#E(), this.#i?.seek(e);
	}
	send(e) {
		if (this.#E(), this.machine === void 0) return;
		let t = this.machine.send(e);
		return t.transition !== void 0 && this.#k(t), t;
	}
	reset() {
		if (this.#E(), this.machine !== void 0) {
			let e = this.machine.reset();
			this.#k(e);
		} else this.restart(!1);
	}
	setTheme(e) {
		this.#E(), this.#t = e, this.#r = this.#D(), this.#O(!1);
	}
	setScene(e, t = {}) {
		this.#E(), this.#e = e, this.machine = Ao(e) && e.machine !== void 0 ? new Ln(e.machine, {
			history: this.#n.history ?? !1,
			...t.initialState === void 0 ? {} : { initialState: t.initialState }
		}) : void 0, this.#l = void 0, this.#M(), this.#r = this.#D(), this.#O(!0);
	}
	setReducedMotion(e) {
		this.#E(), this.#c = e, this.#i?.setReducedMotion(e), this.#F();
	}
	inspect(e) {
		return this.#E(), e === void 0 || this.#H(e === null ? void 0 : this.#V(e)), this.#l;
	}
	resize(e) {
		this.#E();
		let t = Math.max(280, Math.round(e ?? jo(this.element)));
		(t !== this.#s || e !== void 0) && (this.#s = t, this.#r = this.#D(), this.#O(!1), this.#p.emit("resize", {
			width: this.#s,
			layout: this.#r.layoutName
		}));
	}
	on(e, t) {
		return this.#p.on(e, t);
	}
	destroy() {
		if (!this.#u) {
			this.#u = !0, this.#i?.dispose(), this.#i = void 0, this.#a?.dispose(), this.#a = void 0, this.#o?.dispose(), this.#o = void 0, this.#w?.disconnect();
			for (let e of this.#m.splice(0)) e();
			this.#p.emit("destroy", void 0), this.#p.clear(), this.element.replaceChildren(), this.element.classList.remove("kg-figure-host"), this.element.removeAttribute("aria-busy");
		}
	}
	get #T() {
		return this.#r.timeline?.duration ?? 0;
	}
	#E() {
		if (this.#u) throw Error("Kineglyph controller has been destroyed");
	}
	#D() {
		return Si(this.#e, {
			width: this.#s,
			theme: this.#t,
			layout: this.#n.layout ?? "auto",
			...this.machine === void 0 ? {} : { machineState: this.machine.state }
		});
	}
	#O(e) {
		let t = this.#i?.time ?? 0, n = this.#i?.playing ?? !1, r = this.#G();
		this.#i?.dispose(), this.#a?.dispose(), this.#a = void 0, this.#o?.dispose(), this.#o = void 0;
		let i = this.#c || !(this.#n.autoplay ?? !0), a = e ? i ? this.#T : 0 : t, o = Fi(this.#r, a);
		this.stage.innerHTML = zi(o, {
			idPrefix: this.id,
			className: "kg-figure__svg",
			role: "group",
			effects: "enhanced"
		}), this.stage.style.aspectRatio = `${this.#r.width} / ${this.#r.height}`, this.#a = Qa(this.stage, a), this.#o = new po(this.stage, this.#r, {
			...this.#n.liveSurfaces === void 0 ? {} : { renderers: this.#n.liveSurfaces },
			theme: this.#t,
			machineState: this.machine?.state,
			signals: this.machine?.signals ?? {},
			time: a,
			send: (e) => this.send(e),
			...this.#n.onSurfaceError === void 0 ? {} : { onError: this.#n.onSurfaceError }
		}), this.#A(), this.#i = new Ua({
			root: this.stage,
			scene: this.#r,
			reducedMotion: this.#c,
			onFrame: (e) => {
				this.#d = e.time, this.#a?.seek(e.time), this.#o?.update(e), this.#I(), this.#p.emit("frame", e), this.#n.onFrame?.(e);
			},
			onPlaybackChange: (e) => {
				this.#f = e, this.#F(), this.#p.emit("playback", e), this.#n.onPlaybackChange?.(e);
			}
		}), (!e || i && !this.#c) && this.#i.seek(a), this.#N(), this.#F(), this.#U(), this.#W(), this.#p.emit("render", this.#r), e ? (this.#n.autoplay ?? !0) && !this.#c && this.#T > 0 && this.#i.play() : n && !this.#c && this.#i.play(), r !== void 0 && this.#K(r);
	}
	#k(e) {
		this.#r = this.#D(), this.#O(!1);
		for (let t of e.effects) if (t.type === "seek") {
			let e = t.time === "start" ? 0 : t.time === "end" ? this.#T : t.time;
			this.#i?.seek(e);
		}
		if (this.machine !== void 0 && this.#C !== void 0) {
			let t = this.machine.signals, n = [
				t.engine,
				t.insightTitle,
				t.summary
			].filter((e) => typeof e == "string" && e.length > 0).join(" — ");
			this.#C.textContent = n || `State: ${e.next.state}`;
		}
		this.#p.emit("state", {
			step: e,
			scene: this.#r
		}), this.#n.onStateChange?.(e, this.#r);
	}
	#A() {
		let e = this.#t, t = this.#h.style;
		t.setProperty("--kg-shell-background", e.colors.canvas), t.setProperty("--kg-shell-surface", e.colors.surfaceRaised), t.setProperty("--kg-shell-text", e.colors.text), t.setProperty("--kg-shell-muted", e.colors.textMuted), t.setProperty("--kg-shell-border", e.colors.border), t.setProperty("--kg-shell-accent", e.colors.accent), t.setProperty("--kg-shell-radius", `${e.radii.lg}px`), t.setProperty("--kg-shell-font", e.typography.body.family), this.#h.classList.toggle("kg-figure--compact", this.#s < 620), this.#h.setAttribute("aria-label", `${this.#r.title} interactive figure`), this.#h.dataset.layout = this.#r.layoutName ?? this.#r.layout, this.#h.dataset.theme = e.name ?? "custom";
	}
	#j(e) {
		let t = this.#v;
		if (t === void 0) return;
		let n = e.createElement("button");
		n.type = "button", n.className = "kg-figure__play", n.textContent = "Play", n.addEventListener("click", () => {
			this.#f ? this.pause() : this.play();
		});
		let r = e.createElement("button");
		r.type = "button", r.className = "kg-figure__restart", r.textContent = "Restart", r.addEventListener("click", () => this.restart(!1));
		let i = e.createElement("label");
		i.className = "kg-figure__scrubber";
		let a = e.createElement("span");
		a.textContent = "Timeline";
		let o = e.createElement("input");
		o.type = "range", o.min = "0", o.step = "1", o.addEventListener("input", () => this.seek(Number(o.value))), i.append(a, o);
		let s = e.createElement("output");
		t.append(n, r, i, s), t.addEventListener("keydown", (e) => {
			e.key === " " && e.target !== o && (e.target instanceof HTMLButtonElement || (e.preventDefault(), this.#f ? this.pause() : this.play()));
		}), this.#y = n, this.#b = r, this.#x = o, this.#S = s;
	}
	#M() {
		let e = this.#_;
		e !== void 0 && (delete e.dataset.controls, e.replaceChildren());
	}
	#N() {
		let e = this.#_;
		if (e === void 0) return;
		let t = this.#r.controls ?? [];
		if (this.machine === void 0 || t.length === 0) {
			e.hidden = !0, this.#M();
			return;
		}
		e.hidden = !1;
		let n = JSON.stringify(t.map((e) => [
			e.id,
			e.label,
			e.kind ?? "event",
			e.event ?? "",
			e.group ?? "",
			e.description ?? ""
		]));
		if (e.dataset.controls === n && e.childElementCount > 0) {
			this.#P();
			return;
		}
		e.dataset.controls = n;
		let r = e.ownerDocument;
		e.replaceChildren();
		let i = /* @__PURE__ */ new Map();
		for (let e of t) {
			let t = e.group ?? "", n = i.get(t) ?? [];
			n.push(e), i.set(t, n);
		}
		for (let [t, n] of i) {
			let i = r.createElement("div");
			if (i.className = "kg-figure__machine-group", i.setAttribute("role", "group"), t.length > 0) {
				i.setAttribute("aria-label", t);
				let e = r.createElement("span");
				e.className = "kg-figure__machine-label", e.textContent = t, i.append(e);
			}
			for (let e of n) {
				let t = r.createElement("button");
				t.type = "button", t.textContent = e.label, t.dataset.control = e.id, e.description !== void 0 && (t.title = e.description), (e.kind ?? "event") === "reset" ? (t.classList.add("kg-figure__reset"), t.addEventListener("click", () => this.reset())) : t.addEventListener("click", () => {
					e.event !== void 0 && this.send(e.event);
				}), i.append(t);
			}
			e.append(i);
		}
		this.#P();
	}
	#P() {
		let e = this.#_;
		if (e === void 0 || this.machine === void 0) return;
		let t = this.machine.state;
		for (let n of this.#r.controls ?? []) {
			let r = e.querySelector(`[data-control="${Po(n.id)}"]`);
			r !== null && n.activeWhen !== void 0 && r.setAttribute("aria-pressed", On(n.activeWhen, t) ? "true" : "false");
		}
	}
	#F() {
		let e = this.#c || this.#T === 0;
		this.#y !== void 0 && (this.#y.textContent = this.#f ? "Pause" : "Play", this.#y.setAttribute("aria-pressed", this.#f ? "true" : "false"), this.#y.disabled = e), this.#b !== void 0 && (this.#b.disabled = e), this.#x !== void 0 && (this.#x.max = String(Math.max(1, this.#T)), this.#x.disabled = e), this.#I(), this.#P();
	}
	#I() {
		let e = this.#c ? this.#T : this.#d;
		this.#x !== void 0 && (this.#x.value = String(Math.round(e)), this.#x.setAttribute("aria-valuetext", `${Math.round(e)} milliseconds`)), this.#S !== void 0 && (this.#S.textContent = this.#c ? "Reduced motion" : `${(e / 1e3).toFixed(1)}s`);
	}
	#L(e) {
		let t = this.stage, n = (e) => e.target instanceof Element ? e.target.closest("[data-node-id],[data-edge-group]") : null, r = (e) => {
			let t = n(e);
			if (t === null) return;
			let r = t.getAttribute("data-node-id"), i = t.getAttribute("data-edge-group");
			r !== null && this.#B(r) ? this.#H(this.#V(r)) : i !== null && t.getAttribute("role") === "img" && this.#H(this.#V(i));
		}, i = (e) => {
			let t = e instanceof FocusEvent || e instanceof MouseEvent ? e.relatedTarget : null, r = n(e);
			t instanceof Element && t.closest("[data-node-id],[data-edge-group]") === r || e.type === "focusout" && r !== null && !r.matches("[data-node-id]") || this.#H(void 0);
		}, a = (e) => {
			let t = n(e);
			if (t === null) return;
			let r = t.getAttribute("data-activate");
			r !== null && (e instanceof KeyboardEvent && e.key !== "Enter" && e.key !== " " || (e.preventDefault(), this.send(r)));
		};
		t.addEventListener("pointerover", r), t.addEventListener("pointerout", i), t.addEventListener("focusin", r), t.addEventListener("focusout", i);
		let o = (e) => {
			if (!(e instanceof KeyboardEvent) || ![
				"ArrowRight",
				"ArrowDown",
				"ArrowLeft",
				"ArrowUp",
				"Home",
				"End"
			].includes(e.key)) return;
			let t = e.target instanceof Element ? e.target : null, n = t?.closest("[data-focus-group]");
			if (n == null) return;
			let r = No(n);
			if (r.length === 0) return;
			let i = r.findIndex((e) => e === t), a;
			a = e.key === "Home" ? 0 : e.key === "End" ? r.length - 1 : e.key === "ArrowRight" || e.key === "ArrowDown" ? i < 0 ? 0 : (i + 1) % r.length : i < 0 ? r.length - 1 : (i - 1 + r.length) % r.length, e.preventDefault(), r[a]?.focus({ preventScroll: !0 });
		};
		t.addEventListener("click", a), t.addEventListener("keydown", a), t.addEventListener("keydown", o), this.#m.push(() => {
			t.removeEventListener("pointerover", r), t.removeEventListener("pointerout", i), t.removeEventListener("focusin", r), t.removeEventListener("focusout", i), t.removeEventListener("click", a), t.removeEventListener("keydown", a), t.removeEventListener("keydown", o);
		});
	}
	#R(e) {
		if (this.#n.reducedMotion !== void 0) return;
		let t = e.ownerDocument.defaultView;
		if (t === null || typeof t.matchMedia != "function") return;
		let n = t.matchMedia("(prefers-reduced-motion: reduce)"), r = () => {
			this.#u || this.setReducedMotion(n.matches);
		};
		n.addEventListener("change", r), this.#m.push(() => n.removeEventListener("change", r));
	}
	#z(e) {
		let t = e.ownerDocument.defaultView;
		t !== null && typeof t.ResizeObserver == "function" && (this.#w = new t.ResizeObserver((e) => {
			let t = e[0];
			if (t === void 0 || this.#u) return;
			let n = Math.max(280, Math.round(t.contentRect.width));
			n !== this.#s && n > 0 && this.resize(n);
		}), this.#w.observe(e));
	}
	#B(e) {
		let t = this.#r.nodes.find((t) => t.id === e);
		return t !== void 0 && (t.interactive || t.label.length > 0 && t.description !== void 0);
	}
	#V(e) {
		let t = this.#r.nodes.find((t) => t.id === e);
		if (t !== void 0) {
			let n = t.inspect ?? {}, r = t.metadata.role, i = n.role ?? (typeof r == "string" && r.length > 0 ? r : "Element"), a = n.title ?? t.label, o = n.summary ?? t.description;
			return {
				kind: "node",
				id: e,
				role: i,
				title: a,
				...o === void 0 ? {} : { summary: o },
				fields: n.fields ?? [],
				label: a,
				...o === void 0 ? {} : { description: o },
				node: t
			};
		}
		let n = this.#r.edges.find((t) => t.id === e);
		if (n !== void 0) {
			let t = n.label ?? n.description ?? e;
			return {
				kind: "edge",
				id: e,
				role: "Connection",
				title: t,
				...n.description === void 0 ? {} : { summary: n.description },
				fields: [{
					label: "From",
					value: n.from
				}, {
					label: "To",
					value: n.to
				}],
				label: t,
				...n.description === void 0 ? {} : { description: n.description },
				edge: n
			};
		}
	}
	#H(e) {
		this.#l?.id !== e?.id && (this.#l = e, this.#U(), this.#W(), this.#p.emit("inspect", e), this.#n.onInspect?.(e));
	}
	#U() {
		let e = this.machine?.state.selection ?? null;
		for (let t of this.stage.querySelectorAll("[data-node-id]")) {
			let n = t.getAttribute("data-node-id");
			n === this.#l?.id ? t.setAttribute("data-inspected", "true") : t.removeAttribute("data-inspected"), n !== null && n === e ? t.setAttribute("data-selected", "true") : t.removeAttribute("data-selected");
		}
	}
	#W() {
		let e = this.#g;
		if (e === void 0) return;
		let [t, n, r] = e.children, i = this.#l, a = e.ownerDocument;
		if (i === void 0) {
			t && (t.textContent = "Inspect"), n && (n.textContent = this.#r.title), r && (r.textContent = this.#r.description ?? "");
			return;
		}
		if (t && (t.textContent = i.role), n && (n.textContent = i.title), r && (r.replaceChildren(), i.summary !== void 0 && i.summary.length > 0 && r.append(a.createTextNode(i.summary)), i.fields.length > 0)) {
			let e = a.createElement("dl");
			e.className = "kg-figure__fields";
			for (let t of i.fields) {
				let n = a.createElement("dt");
				n.textContent = t.label;
				let r = a.createElement("dd");
				r.textContent = t.value, e.append(n, r);
			}
			r.append(e);
		}
	}
	#G() {
		let e = this.element.ownerDocument.activeElement;
		if (!(!(e instanceof Element) || !this.stage.contains(e))) return e.closest("[data-node-id]")?.getAttribute("data-node-id") ?? void 0;
	}
	#K(e) {
		let t = this.stage.querySelector(`[data-node-id="${Po(e)}"]`);
		t !== null && typeof t.focus == "function" && t.focus({ preventScroll: !0 });
	}
}, wo = /* @__PURE__ */ new Map(), To = /* @__PURE__ */ new Map();
function Eo(e, t) {
	wo.set(e, t);
}
function Do(e, t) {
	To.set(e, t);
}
function Oo(e = {}) {
	let t = e.root ?? (typeof document > "u" ? {} : document);
	if (typeof t.querySelectorAll != "function") return [];
	let n = [];
	for (let r of t.querySelectorAll(e.selector ?? "[data-kineglyph]")) {
		if (r.dataset.kineglyphMounted === "true") continue;
		let t = r.dataset.kineglyph ?? "", i = e.scenes?.[t] ?? wo.get(t);
		if (i === void 0) {
			r.setAttribute("data-kineglyph-error", `unknown scene "${t}"`);
			continue;
		}
		let a = r.dataset.theme, o = a === void 0 ? void 0 : e.themes?.[a] ?? To.get(a), s = r.dataset.layout, c = r.dataset.width === void 0 ? void 0 : Number(r.dataset.width), l = e.mountOptions?.(r, t) ?? {}, u = xo(r, {
			scene: i,
			...o === void 0 ? {} : { theme: o },
			...s === void 0 ? {} : { layout: s },
			...c === void 0 || !Number.isFinite(c) ? {} : { width: c },
			autoplay: r.dataset.autoplay !== "false",
			controls: r.dataset.controls !== "false",
			readout: r.dataset.readout !== "false",
			...r.dataset.reducedMotion === void 0 ? {} : { reducedMotion: r.dataset.reducedMotion === "true" },
			...r.dataset.idPrefix === void 0 ? {} : { idPrefix: r.dataset.idPrefix },
			...l
		});
		r.dataset.kineglyphMounted = "true", u.on("destroy", () => {
			delete r.dataset.kineglyphMounted;
		}), n.push(u);
	}
	return n;
}
function ko(e, t, n = {}) {
	let r = e.ownerDocument.defaultView?.IntersectionObserver;
	if (typeof r != "function") return n.fallbackImmediately !== !1 && t(), () => void 0;
	let i = !1, a = new r((e) => {
		if (e.some((e) => e.isIntersecting)) {
			if (n.once !== !1) {
				if (i) return;
				i = !0, a.disconnect();
			}
			t();
		}
	}, {
		threshold: n.threshold ?? .06,
		rootMargin: n.rootMargin ?? "0px 0px -10% 0px"
	});
	return a.observe(e), () => a.disconnect();
}
function Ao(e) {
	return e.schemaVersion === 2;
}
function jo(e) {
	let t = e.getBoundingClientRect();
	if (t.width > 0) return t.width;
	let n = e.parentElement, r = n === null ? 0 : n.getBoundingClientRect().width;
	return r > 0 ? r : 960;
}
function Mo(e) {
	let t = e.ownerDocument.defaultView;
	return t === null || typeof t.matchMedia != "function" ? !1 : t.matchMedia("(prefers-reduced-motion: reduce)").matches;
}
function No(e) {
	return [...e.querySelectorAll("[data-node-id][tabindex]")].filter((t) => {
		if (t === e || t.hasAttribute("data-focus-group") || (t.parentElement?.closest("[data-focus-group]") ?? null) !== e) return !1;
		for (let n = t; n !== null && n !== e; n = n.parentElement) if (n.getAttribute("data-hidden") === "true" || n.getAttribute("display") === "none" || n.getAttribute("aria-hidden") === "true" || n.hasAttribute("inert") || n.hasAttribute("hidden")) return !1;
		return !0;
	});
}
function Po(e) {
	return typeof CSS < "u" && typeof CSS.escape == "function" ? CSS.escape(e) : e.replace(/["\\]/g, "\\$&");
}
//#endregion
//#region ../plot/dist/data.js
function Fo(e, t = "item") {
	let n = e.trim().replace(/[^A-Za-z0-9_.-]+/g, "-").replace(/^-+|-+$/g, "");
	return n.length > 0 ? n : t;
}
function Io(e, t = "series") {
	let n = /* @__PURE__ */ new Map();
	return e.map((e, r) => {
		let i = Fo(e, `${t}-${r + 1}`), a = n.get(i) ?? 0;
		return n.set(i, a + 1), a === 0 ? i : `${i}-${a + 1}`;
	});
}
function Lo(e) {
	return typeof e == "number" && Number.isFinite(e) ? e : null;
}
function Ro(e) {
	if (typeof e == "number" && Number.isFinite(e) || typeof e == "string") return e;
	if (typeof e == "boolean") return String(e);
}
function zo(e) {
	return typeof e == "string" ? e : typeof e == "number" || typeof e == "boolean" ? String(e) : "";
}
function Bo(e) {
	return !Array.isArray(e);
}
function Vo(e, t, n) {
	let r = [], i = 0, a = 0, o = (e, t, n, o, s) => {
		let c = Ro(e);
		if (c === void 0) {
			i += 1;
			return;
		}
		let l = Lo(t);
		l === null && t != null && (a += 1), r.push({
			x: c,
			y: l,
			...typeof n == "string" && n.length > 0 ? { tone: n } : {},
			...typeof o == "string" && o.length > 0 ? { label: o } : {},
			...typeof s == "string" && s.length > 0 ? { description: s } : {}
		});
	};
	if (Bo(e)) for (let t of e.rows) o(t[e.x], t[e.y], e.tone === void 0 ? void 0 : t[e.tone], e.label === void 0 ? void 0 : t[e.label], e.description === void 0 ? void 0 : t[e.description]);
	else for (let t of e) o(t.x, t.y, t.tone, t.label, t.description);
	return i > 0 && n.push({
		severity: "warning",
		code: "invalid-x",
		message: `series ${t}: ${i} data point(s) without a usable x value were skipped`
	}), a > 0 && n.push({
		severity: "warning",
		code: "non-numeric-value",
		message: `series ${t}: ${a} non-numeric y value(s) were treated as missing`
	}), r;
}
function Ho(e) {
	let t = /* @__PURE__ */ new Set(), n = [];
	for (let r of e) {
		if (r == null) continue;
		let e = zo(r);
		t.has(e) || (t.add(e), n.push(e));
	}
	return n;
}
function Uo(e, t) {
	let n = e === !1 || e === void 0 ? {} : e;
	if ((n.type ?? t) === "band") {
		let e = Array.isArray(n.domain) ? n.domain.map(String) : void 0;
		return {
			type: "band",
			...e === void 0 ? {} : { domain: e },
			...n.padding === void 0 ? {} : { padding: n.padding },
			...n.label === void 0 ? {} : { label: n.label }
		};
	}
	let r = n.domain, i = Array.isArray(r) && r.length === 2 && typeof r[0] == "number" && typeof r[1] == "number" ? [r[0], r[1]] : r === "auto" || r === "auto-zero" ? r : void 0;
	return {
		type: "linear",
		...i === void 0 ? {} : { domain: i },
		...n.nice === void 0 ? {} : { nice: n.nice },
		...n.ticks === void 0 ? {} : { ticks: n.ticks },
		...n.format === void 0 ? {} : { format: n.format },
		...n.label === void 0 ? {} : { label: n.label }
	};
}
function Wo(e) {
	return e === !1 || e === void 0 ? e : {
		...e.label === void 0 ? {} : { label: e.label },
		...e.hidden === void 0 ? {} : { hidden: e.hidden },
		...e.labelEvery === void 0 ? {} : { labelEvery: e.labelEvery },
		...e.format === void 0 ? {} : { format: e.format }
	};
}
function Go(e) {
	switch (e) {
		case "bar":
		case "grouped-bar":
		case "stacked-bar": return "bar";
		case "line":
		case "sparkline": return "line";
		case "area": return "area";
		case "dot": return "scatter";
	}
}
function Ko(e, t, n) {
	let r = e, i = Ho(r.map((e) => e[t.row])), a = Ho(r.map((e) => e[t.column])), o = new Map(i.map((e, t) => [e, t])), s = new Map(a.map((e, t) => [e, t])), c = i.map(() => a.map(() => null)), l = /* @__PURE__ */ new Set(), u = 0;
	for (let e of r) {
		let n = e[t.row], r = e[t.column];
		if (n == null || r == null) continue;
		let i = o.get(zo(n)), a = s.get(zo(r));
		if (i === void 0 || a === void 0) continue;
		let d = `${i}:${a}`;
		l.has(d) && (u += 1), l.add(d);
		let f = c[i];
		f !== void 0 && (f[a] = Lo(e[t.value]));
	}
	return u > 0 && n.push({
		severity: "warning",
		code: "duplicate-cell",
		message: `heatmap: ${u} duplicate row/column pair(s); the last value wins`
	}), {
		rows: i,
		columns: a,
		values: c,
		...t.domain === void 0 ? {} : { domain: t.domain },
		...t.tone === void 0 ? {} : { tone: t.tone },
		...t.negativeTone === void 0 ? {} : { negativeTone: t.negativeTone },
		...t.cellLabels === void 0 ? {} : { cellLabels: t.cellLabels },
		...t.format === void 0 ? {} : { format: t.format },
		rowLabel: t.row,
		columnLabel: t.column
	};
}
function qo(e) {
	return e === void 0 ? [] : Array.isArray(e) ? [...e] : [e];
}
function Jo(e, t) {
	let n = [], r = e, i = t, a = {
		...t.title === void 0 ? {} : { title: t.title },
		...t.description === void 0 ? {} : { description: t.description },
		...t.legend === void 0 ? {} : { legend: t.legend },
		...t.minimal === void 0 ? {} : { minimal: t.minimal },
		...t.height === void 0 ? {} : { height: t.height }
	}, o = {
		...a,
		...i.grid === void 0 ? {} : { grid: i.grid },
		...i.annotations === void 0 ? {} : { annotations: i.annotations },
		...i.valueLabels === void 0 ? {} : { valueLabels: i.valueLabels },
		...i.orientation === void 0 ? {} : { orientation: i.orientation },
		...i.stack === void 0 ? {} : { stack: i.stack }
	}, s = Wo(t.axes?.x), c = Wo(t.axes?.y), l = {
		...s === void 0 ? {} : { x: s },
		...c === void 0 ? {} : { y: c }
	}, u = (e) => Object.keys(l).length === 0 ? e : {
		...e,
		axes: l
	}, d = qo(t.marks), f = d.find((e) => e.kind === "heatmap");
	if (f !== void 0) return d.length > 1 && n.push({
		severity: "warning",
		code: "heatmap-layers",
		message: "heatmaps cannot be layered with other marks; extra layers were ignored"
	}), {
		spec: u({
			...a,
			heatmap: Ko(e, f, n)
		}),
		seriesKeys: [{
			key: "heatmap",
			id: "heatmap"
		}],
		diagnostics: n
	};
	let p = d.filter((e) => e.kind !== "heatmap"), m = t.x, h = t.y === void 0 ? [] : typeof t.y == "string" ? [t.y] : [...t.y];
	if (h.length === 0) return n.push({
		severity: "error",
		code: "missing-channel",
		message: "plot(rows, options) needs a \"y\" channel"
	}), {
		spec: u({
			...o,
			series: []
		}),
		seriesKeys: [],
		diagnostics: n
	};
	let g = m === void 0 ? r.map((e, t) => t) : r.map((e) => e[m]).filter((e) => e != null), _ = g.length > 0 && g.every((e) => typeof e == "number"), v = p.length > 0 ? p : [{ kind: _ ? "line" : "bar" }], y = i.stack ?? v.some((e) => e.kind === "stacked-bar"), b = t.minimal ?? v.every((e) => e.kind === "sparkline"), x = v.find((e) => e.padding !== void 0)?.padding, S = (e, n, r) => {
		let i = m === void 0 ? r : e[m], a = t.tone === void 0 ? void 0 : e[t.tone], o = t.label === void 0 ? void 0 : e[t.label];
		return {
			x: typeof i == "number" && Number.isFinite(i) ? i : zo(i),
			y: Lo(e[n]),
			...typeof a == "string" ? { tone: a } : {},
			...typeof o == "string" ? { label: o } : {}
		};
	}, C = [], w = t.series;
	if (w !== void 0) {
		let e = Ho(r.map((e) => e[w]));
		for (let t of e) {
			let e = r.filter((e) => {
				let n = e[w];
				return n != null && zo(n) === t;
			});
			for (let n of h) {
				let r = h.length === 1 ? t : `${t} ${n}`;
				C.push({
					key: r,
					label: r,
					data: e.map((e, t) => S(e, n, t))
				});
			}
		}
	} else for (let e of h) C.push({
		key: e,
		label: e,
		data: r.map((t, n) => S(t, e, n))
	});
	let T = Io(C.map((e) => e.key)), E = [];
	for (let e of v) {
		let t = {
			...e.tone === void 0 ? {} : { tone: e.tone },
			...e.fill === void 0 ? {} : { fill: e.fill },
			...e.fillOpacity === void 0 ? {} : { fillOpacity: e.fillOpacity },
			...e.curve === void 0 ? {} : { curve: e.curve },
			...e.dash === void 0 ? {} : { dash: e.dash },
			...e.pointRadius === void 0 ? {} : { pointRadius: e.pointRadius },
			...e.interactive === void 0 ? {} : { interactive: e.interactive }
		};
		C.forEach((n, r) => {
			let a = i.seriesBindings?.[n.key];
			E.push({
				id: T[r] ?? `series-${r + 1}`,
				label: n.label,
				mark: Go(e.kind),
				data: n.data,
				...a === void 0 ? {} : { bind: a },
				...t
			});
		});
	}
	let D = Uo(t.axes?.x, _ ? "linear" : "band");
	return {
		spec: u({
			...o,
			...y ? { stack: !0 } : {},
			...b ? { minimal: !0 } : {},
			series: E,
			x: D.type === "band" && x !== void 0 && D.padding === void 0 ? {
				...D,
				padding: x
			} : D,
			y: Uo(t.axes?.y, "linear")
		}),
		seriesKeys: C.map((e, t) => ({
			key: e.key,
			id: T[t] ?? ""
		})),
		diagnostics: n
	};
}
//#endregion
//#region ../plot/dist/scales.js
function Yo(e, t = [0, 1]) {
	let [n, r] = e, [i, a] = t, o = r - n;
	return {
		domain: [n, r],
		range: [i, a],
		map: (e) => o === 0 ? (i + a) / 2 : i + (e - n) / o * (a - i),
		invert: (e) => a - i === 0 ? n : n + (e - i) / (a - i) * o
	};
}
function Xo(e, t = [0, 1], n = .25) {
	let [r, i] = t, a = e.length, o = ss(n, 0, .9), s = a === 0 ? 0 : (i - r) / a, c = s * (1 - o), l = /* @__PURE__ */ new Map();
	return e.forEach((e, t) => {
		l.has(e) || l.set(e, t);
	}), {
		domain: e,
		range: [r, i],
		padding: o,
		step: s,
		bandwidth: c,
		band: (e) => {
			let t = typeof e == "number" ? e : l.get(e);
			if (t === void 0 || t < 0 || t >= a) return;
			let n = r + t * s + s * o / 2;
			return {
				start: n,
				end: n + c,
				width: c,
				center: n + c / 2
			};
		},
		index: (e) => l.get(e) ?? -1
	};
}
function Zo(e, t, n) {
	let r = (t - e) / Math.max(1, n), i = Math.floor(Math.log10(r)), a = r / 10 ** i, o = a >= Math.sqrt(50) ? 10 : a >= Math.sqrt(10) ? 5 : a >= Math.SQRT2 ? 2 : 1;
	return i >= 0 ? o * 10 ** i : -(10 ** -i) / o;
}
function Qo(e, t, n = 5) {
	if (!(t > e)) return 1;
	let r = Zo(e, t, n);
	return r < 0 ? 1 / -r : r;
}
function $o(e, t, n = 5) {
	if (!Number.isFinite(e) || !Number.isFinite(t)) return [];
	if (e === t) return [e];
	let r = t < e, i = r ? t : e, a = r ? e : t, o = Zo(i, a, Math.max(1, Math.floor(n)));
	if (!Number.isFinite(o) || o === 0) return [];
	let s = [];
	if (o > 0) {
		let e = Math.ceil(i / o), t = Math.floor(a / o);
		for (let n = e; n <= t; n += 1) s.push(n * o);
	} else {
		let e = -o, t = Math.ceil(i * e), n = Math.floor(a * e);
		for (let r = t; r <= n; r += 1) s.push(r / e);
	}
	let c = s.map((e) => Object.is(e, -0) ? 0 : e);
	return r ? c.reverse() : c;
}
function es(e, t, n = 5) {
	let r = e, i = t;
	if (!(i > r)) return [r, i];
	for (let e = 0; e < 2; e += 1) {
		let e = Zo(r, i, Math.max(1, Math.floor(n)));
		if (!Number.isFinite(e) || e === 0) break;
		if (e > 0) r = Math.floor(r / e) * e, i = Math.ceil(i / e) * e;
		else {
			let t = -e;
			r = Math.floor(r * t) / t, i = Math.ceil(i * t) / t;
		}
	}
	return [Object.is(r, -0) ? 0 : r, Object.is(i, -0) ? 0 : i];
}
function ts(e) {
	if (!Number.isFinite(e) || e === 0) return 0;
	for (let t = 0; t <= 6; t += 1) if (Math.abs(Math.round(e * 10 ** t) / 10 ** t - e) < 1e-9) return t;
	return 6;
}
function ns(e) {
	let t = "";
	for (let n = 0; n < e.length; n += 1) {
		let r = e.length - n;
		t += e[n] ?? "", r > 1 && r % 3 == 1 && (t += ",");
	}
	return t;
}
function rs(e) {
	return e.includes(".") ? e.replace(/0+$/, "").replace(/\.$/, "") : e;
}
function is(e, t = {}) {
	if (!Number.isFinite(e)) return "–";
	let n = e < 0, r = Math.abs(e), i = "";
	t.compact === !0 && (r >= 1e9 ? (r /= 1e9, i = "B") : r >= 1e6 ? (r /= 1e6, i = "M") : r >= 1e3 && (r /= 1e3, i = "k"));
	let a;
	a = t.digits === void 0 ? i === "" ? t.step === void 0 ? rs(r.toFixed(3)) : r.toFixed(ts(t.step)) : rs(r.toFixed(1)) : r.toFixed(Math.max(0, Math.min(6, t.digits)));
	let [o = "0", s] = a.split("."), c = t.thousands === !0 || t.thousands === void 0 && i === "" && r >= 1e4 ? ns(o) : o, l = s === void 0 ? c : `${c}.${s}`, u = /^[0.]*$/.test(l.replace(/,/g, ""));
	return `${n && !u ? "-" : ""}${t.prefix ?? ""}${l}${i}${t.suffix ?? ""}`;
}
function as(e, t = {}) {
	let n = t.domain;
	if (Array.isArray(n)) {
		let [e, t] = n;
		if (Number.isFinite(e) && Number.isFinite(t) && e !== t) return e < t ? [e, t] : [t, e];
	}
	let r = [];
	for (let t of e) typeof t == "number" && Number.isFinite(t) && r.push(t);
	for (let e of t.include ?? []) Number.isFinite(e) && r.push(e);
	if (r.length === 0) return [0, 1];
	let i = Math.min(...r), a = Math.max(...r);
	if (n !== "auto" && (i = Math.min(i, 0), a = Math.max(a, 0)), i === a) {
		if (i === 0) a = 1;
		else {
			let e = Math.abs(i);
			i -= e, a += e;
		}
	}
	let o = t.headroom ?? 0;
	if (o > 0) {
		let e = a - i;
		a > 0 && (a += e * o), i < 0 && (i -= e * o);
	}
	return t.nice === !1 ? [i, a] : es(i, a, t.ticks ?? 5);
}
function os(e) {
	let t = Math.max(0, ...e.map((e) => e.length)), n = Array(t).fill(0), r = Array(t).fill(0);
	return e.map((e) => {
		let i = [];
		for (let a = 0; a < t; a += 1) {
			let t = e[a];
			if (typeof t != "number" || !Number.isFinite(t)) {
				i.push(null);
				continue;
			}
			if (t >= 0) {
				let e = n[a] ?? 0;
				n[a] = e + t, i.push({
					start: e,
					end: e + t
				});
			} else {
				let e = r[a] ?? 0;
				r[a] = e + t, i.push({
					start: e,
					end: e + t
				});
			}
		}
		return i;
	});
}
function ss(e, t, n) {
	return Math.min(n, Math.max(t, e));
}
//#endregion
//#region ../plot/dist/types.js
var cs = {
	wide: 240,
	compact: 200,
	narrow: 160
}, ls = {
	wide: 48,
	compact: 40,
	narrow: 32
}, us = [
	"chart1",
	"chart2",
	"chart3",
	"chart4",
	"chart5",
	"chart6"
], ds = "plot", fs = 900, ps = 8, ms = 12, hs = 16, gs = 28, _s = {
	wide: 200,
	compact: 140,
	narrow: 100
}, vs = 1.25, ys = 18, bs = 20, xs = {
	wide: 800,
	compact: 460,
	narrow: 250
}, Ss = {
	wide: 16,
	compact: 8,
	narrow: 4
}, Cs = 200, ws = 8, Ts = .15, Es = 4, Ds = 5;
function Q(e) {
	return {
		wide: e("wide"),
		compact: e("compact"),
		narrow: e("narrow")
	};
}
function Os(e) {
	return e.wide || e.compact || e.narrow;
}
function ks(e) {
	let t = (e) => JSON.stringify(e);
	if (t(e.wide) === t(e.compact) && t(e.compact) === t(e.narrow)) return e.wide;
	let n = { wide: e.wide };
	return t(e.compact) !== t(e.wide) && (n.compact = e.compact), t(e.narrow) !== t(e.compact) && (n.narrow = e.narrow), n;
}
function As(e) {
	if (!Os(e)) return;
	if (e.wide && e.compact && e.narrow) return !0;
	let t = {};
	return e.wide && (t.wide = !0), e.compact !== e.wide && (t.compact = e.compact), e.narrow !== e.compact && (t.narrow = e.narrow), t;
}
function js(e) {
	let t = As(e);
	return t === void 0 ? {} : { hidden: t };
}
function Ms(e, t, n) {
	return vt(e, t) ?? n;
}
function Ns(e) {
	return `${Math.round(ss(e, 0, 1) * 1e6) / 1e4}%`;
}
function $(e) {
	let t = Math.round(e * 1e6) / 1e6;
	return Object.is(t, -0) ? 0 : t;
}
function Ps(e) {
	let t = Mt.typography[e];
	return {
		family: t.family,
		size: t.size,
		weight: t.weight,
		lineHeight: t.lineHeight,
		...t.letterSpacing === void 0 ? {} : { letterSpacing: t.letterSpacing }
	};
}
var Fs = Ps("caption");
function Is(e) {
	return Ot(e, Fs) * vs;
}
function Ls(e) {
	return us[e % us.length] ?? "chart1";
}
function Rs(e, t, n = `${t}s`) {
	return `${e} ${e === 1 ? t : n}`;
}
function zs(e, t, n) {
	return e <= 1 ? 0 : Math.min(n, t / (e - 1));
}
function Bs(e, t, n, r, i = "easeOut") {
	let a = [], o = Math.round(e * 1e3) / 1e3, s = Math.round(Math.max(t, e + 1) * 1e3) / 1e3;
	return o > 0 && a.push({
		time: 0,
		value: n
	}), a.push({
		time: o,
		value: n
	}), a.push({
		time: s,
		value: r,
		easing: i
	}), a;
}
function Vs(e, t, n) {
	return {
		id: `${e}:${t}`,
		target: e,
		property: t,
		keyframes: n
	};
}
function Hs(e, t) {
	return t === "horizontal" ? e >= 1 - 1e-9 ? "bottom-left" : "top-left" : e >= 1 - 1e-9 ? "top-right" : "top-left";
}
function Us(e, t, n, r, i = "border") {
	return {
		id: e,
		type: "rect",
		...n === "horizontal" ? {
			position: {
				x: 0,
				y: $(t),
				anchor: Hs(t, n)
			},
			width: "100%",
			height: 1
		} : {
			position: {
				x: $(t),
				y: 0,
				anchor: Hs(t, n)
			},
			width: 1,
			height: "100%"
		},
		fill: i,
		stroke: "none",
		radius: 0,
		...r === void 0 ? {} : { hidden: r }
	};
}
function Ws(e, t, n = !1) {
	return {
		id: e,
		type: "group",
		layout: "coordinates",
		position: {
			x: 0,
			y: 0
		},
		width: "100%",
		height: "100%",
		...n ? { allowOverflow: !0 } : {},
		children: t
	};
}
function Gs(e, t = {}) {
	let n = t.id === void 0 || t.id.length === 0 ? ds : t.id, r = [], i = e.minimal === !0, a = Q((t) => Math.max(8, vt(e.height, t) ?? (i ? ls[t] : cs[t]))), o = {
		p: n,
		spec: e,
		diagnostics: r,
		minimal: i,
		horizontal: e.orientation === "horizontal",
		heights: a,
		duration: Math.max(1, t.duration ?? fs),
		motion: t.motion ?? "auto",
		easing: t.easing ?? "easeOut"
	};
	return e.heatmap === void 0 ? Qs(o) : nc(o, e.heatmap);
}
function Ks(e) {
	switch (e.mark) {
		case "bar": return "bar";
		case "line": return "line";
		case "area": return "area";
		default: return "point";
	}
}
function qs(e) {
	let t = e.spec.series ?? [], n = [], r = /* @__PURE__ */ new Map();
	for (let e of t) {
		let t = r.get(e.id);
		t === void 0 ? (r.set(e.id, [e]), n.push(e.id)) : t.push(e);
	}
	let i = Io(n);
	return n.map((t, n) => {
		let a = r.get(t) ?? [], o = [], s = /* @__PURE__ */ new Set(), c = a[0]?.tone ?? Ls(n);
		for (let n of a) {
			let r = Ks(n);
			if (s.has(r)) {
				e.diagnostics.push({
					severity: "error",
					code: "duplicate-layer",
					message: `series ${t} declares the ${r} mark twice; the later layer was skipped`
				});
				continue;
			}
			s.add(r), o.push({
				spec: n,
				mark: r,
				tone: n.tone ?? c,
				fill: n.fill,
				fillOpacity: n.fillOpacity,
				points: Vo(n.data, n.id, e.diagnostics),
				pointRadius: Math.max(0, n.pointRadius ?? (e.minimal ? 0 : Es))
			});
		}
		let l = o[0], u = o.find((e) => e.mark === "bar") ?? o.find((e) => e.mark === "point") ?? l, d = u !== void 0 && (u.mark === "bar" || u.mark === "point") ? "marks" : "series", f = a.find((e) => e.interactive !== void 0)?.interactive ?? d, p = Math.max(0, ...o.map((e) => e.points.length)), m = f === "marks" && p > 60 ? "series" : f;
		f === "marks" && m === "series" && e.diagnostics.push({
			severity: "warning",
			code: "interactive-cap",
			message: `series ${t} has ${p} marks; inspecting the series as a whole (cap 60)`
		});
		let h = (e) => a.find((t) => t.bind?.[e] !== void 0)?.bind?.[e], g = h("hidden"), _ = h("opacity"), v = h("highlight"), y = {
			...g === void 0 ? {} : { hidden: g },
			..._ === void 0 ? {} : { opacity: _ },
			...v === void 0 ? {} : { highlight: v }
		};
		return {
			key: t,
			id: i[n] ?? `series-${n + 1}`,
			index: n,
			label: a[0]?.label ?? t,
			description: a.find((e) => e.description !== void 0)?.description,
			tone: c,
			layers: o,
			points: l?.points ?? [],
			interactive: m,
			bind: Object.keys(y).length === 0 ? void 0 : y
		};
	});
}
function Js(e, t) {
	if (t !== void 0) return t.map(String);
	let n = /* @__PURE__ */ new Set(), r = [];
	for (let t of e) for (let e of t.layers) for (let t of e.points) {
		let e = String(t.x);
		n.has(e) || (n.add(e), r.push(e));
	}
	return r;
}
function Ys(e) {
	let t = e?.ticks;
	if (Array.isArray(t)) return {
		counts: Q(() => Ds),
		explicit: t
	};
	let n = t;
	return {
		counts: Q((e) => Math.max(2, Math.floor(Ms(n, e, Ds)))),
		explicit: void 0
	};
}
function Xs(e, t, n) {
	let { counts: r, explicit: i } = Ys(t), a = Math.max(r.wide, r.compact, r.narrow), o = as(e, {
		domain: t?.domain,
		nice: t?.nice ?? !0,
		ticks: a,
		headroom: n,
		...i === void 0 ? {} : { include: i }
	}), s = Yo(o, [0, 1]), c = (e) => e >= o[0] - 1e-9 && e <= o[1] + 1e-9, l = Q((e) => i === void 0 ? $o(o[0], o[1], r[e]) : i.filter((e) => Number.isFinite(e) && c(e)));
	return {
		domain: o,
		scale: s,
		ticks: l,
		step: l.wide.length >= 2 ? Math.abs((l.wide[1] ?? 0) - (l.wide[0] ?? 0)) : Qo(o[0], o[1], r.wide)
	};
}
function Zs(e, t, n, r) {
	if (e <= 1) return 1;
	let i = e > r ? Math.ceil(e / r) : 1, a = n / e, o = Math.max(1, Math.ceil((t + 6) / Math.max(1, a)));
	return Math.max(i, o);
}
function Qs(e) {
	let { p: t, spec: n, diagnostics: r, horizontal: i } = e, a = qs(e), o = a.flatMap((e) => e.layers), s = n.x, c = n.y, l = o.some((e) => e.points.some((e) => typeof e.x == "string")), u = o.some((e) => e.mark === "bar"), d = s?.type ?? (l || u ? "band" : "linear"), f = n.stack === !0, p = e.minimal ? !1 : n.valueLabels ?? !1, m = o.some((e) => e.mark === "point" || e.pointRadius > 0), h = Math.max(0, ...o.map((e) => e.pointRadius)), g = d === "band" ? Js(a, s?.type === "band" ? s.domain : void 0) : [], _ = s?.type === "band" ? ss(s.padding ?? .25, 0, .9) : .25, v = d === "band" ? Xo(g, [0, 1], _) : void 0;
	if (d === "band") for (let e of o) {
		let t = /* @__PURE__ */ new Set(), n = 0;
		for (let r of e.points) {
			let e = String(r.x);
			t.has(e) && (n += 1), t.add(e);
		}
		n > 0 && r.push({
			severity: "warning",
			code: "duplicate-category",
			message: `series ${e.spec.id} repeats ${Rs(n, "category", "categories")}; the last value wins`
		});
	}
	let y = (e) => {
		let t = /* @__PURE__ */ new Map();
		for (let n of e.points) t.set(String(n.x), n.y);
		return g.map((e) => t.get(e) ?? null);
	}, b = f && d === "band" ? o.filter((e) => e.mark === "bar" || e.mark === "area") : [], x = /* @__PURE__ */ new Map();
	if (b.length > 0) {
		let e = os(b.map(y));
		b.forEach((t, n) => x.set(t, e[n] ?? []));
	}
	let S = c?.type === "linear" ? c : void 0, C = [];
	for (let e of o) {
		let t = x.get(e);
		if (t !== void 0) for (let e of t) e !== null && C.push(e.start, e.end);
		else for (let t of e.points) C.push(t.y);
	}
	let w = p === !0, T = e.heights[w ? "narrow" : "compact"], E = Xs(C, S, p === !1 ? 0 : i ? Math.min(.4, (Math.max(0, ...C.map((e) => e === null ? 0 : Is(String(e)))) + 12) / xs[w ? "narrow" : "compact"]) : Math.min(.4, (ys + (m ? h + 4 : 4)) / T)), D = {
		...S?.format ?? {},
		...n.axes?.y === !1 ? {} : n.axes?.y?.format ?? {}
	}, O = (e) => is(e, {
		...D,
		step: E.step
	}), k = (e) => is(e, D), A = s?.type === "linear" ? s : void 0, j = [];
	if (d === "linear") for (let e of o) for (let t of e.points) typeof t.x == "number" && j.push(t.x);
	let M = d === "linear" ? Xs(j, {
		...A ?? { type: "linear" },
		domain: A?.domain ?? "auto"
	}, 0) : void 0, N = {
		...A?.format ?? {},
		...n.axes?.x === !1 ? {} : n.axes?.x?.format ?? {}
	}, P = (e) => is(e, {
		...N,
		step: M?.step ?? 1
	}), ee = d === "band" ? g : M?.domain ?? [0, 1], te = (e) => v === void 0 ? typeof e == "number" ? M?.scale.map(e) : void 0 : v.band(String(e))?.center, F = (e) => E.scale.map(e), ne = (e, t) => i ? {
		x: t,
		y: d === "band" ? e : 1 - e
	} : {
		x: e,
		y: 1 - t
	}, re = (e, t, n, r) => {
		let a = Math.min(e, t), o = Math.max(e, t), s = Math.min(n, r), c = Math.max(n, r);
		return i ? {
			x: s,
			y: d === "band" ? a : 1 - o,
			w: c - s,
			h: o - a
		} : {
			x: a,
			y: 1 - c,
			w: o - a,
			h: c - s
		};
	}, ie = ss(0, E.domain[0], E.domain[1]), ae = n.axes ?? {}, oe = e.minimal ? !1 : ae.x ?? {}, se = e.minimal ? !1 : ae.y ?? {}, ce = v === void 0 ? [] : g.map((e) => ({
		position: v.band(e)?.center ?? 0,
		text: e,
		value: e
	})), le = Q(d === "band" ? () => ce : (e) => (M?.ticks[e] ?? []).map((e) => ({
		position: M?.scale.map(e) ?? 0,
		text: P(e),
		value: e
	}))), ue = Q((e) => E.ticks[e].map((e) => ({
		position: E.scale.map(e),
		text: O(e),
		value: e
	}))), de = (e) => Q((t) => e !== !1 && !Ms(e.hidden, t, !1)), fe = (e, t) => Q((n) => {
		let r = Math.max(0, ...e[n].map((e) => Is(e.text)));
		return Math.ceil(ss(r + ps + 4, gs, t ? _s[n] : 96));
	}), pe = {
		channel: "x",
		side: i ? "left" : "bottom",
		kind: d,
		title: (oe === !1 ? void 0 : oe.label) ?? s?.label,
		shown: de(oe),
		ticks: le,
		labelEvery: oe === !1 ? void 0 : oe.labelEvery,
		gutter: fe(le, d === "band")
	}, me = {
		channel: "y",
		side: i ? "bottom" : "left",
		kind: "linear",
		title: (se === !1 ? void 0 : se.label) ?? c?.label,
		shown: de(se),
		ticks: ue,
		labelEvery: se === !1 ? void 0 : se.labelEvery,
		gutter: fe(ue, !1)
	}, I = i ? pe : me, L = i ? me : pe, he = {
		x: `${t}:axis:x`,
		y: `${t}:axis:y`
	}, ge = e.minimal ? Math.max(3, h + 1) : Math.max(ms, h + 2), _e = ge, ve = e.minimal ? ge : Math.max(hs, h + 2), ye = Q((e) => L.shown[e] ? 0 : ge), be = Q((e) => I.shown[e] ? 0 : ge), xe = Q((e) => I.shown[e] ? I.gutter[e] : 0), Se = Q((t) => e.heights[t] + _e + ye[t]), Ce = [], we = {}, Te = /* @__PURE__ */ new Map(), Ee = n.grid ?? (e.minimal ? "none" : "auto"), De = Ee === "both" || Ee === "y" || Ee === "auto", Oe = Ee === "both" || Ee === "x", ke = [], Ae = (e) => [.../* @__PURE__ */ new Set([
		...e.wide,
		...e.compact,
		...e.narrow
	])].sort((e, t) => e - t);
	Ee !== "none" && (De && Ae(E.ticks).forEach((e, n) => {
		let r = As(Q((t) => !E.ticks[t].includes(e)));
		ke.push(Us(`${t}:grid:${n}`, i ? F(e) : 1 - F(e), i ? "vertical" : "horizontal", r));
	}), Oe && M !== void 0 && Ae(M.ticks).forEach((e, n) => {
		let r = As(Q((t) => !M.ticks[t].includes(e)));
		ke.push(Us(`${t}:grid:x:${n}`, i ? 1 - M.scale.map(e) : M.scale.map(e), i ? "horizontal" : "vertical", r));
	}), E.domain[0] < 0 && E.domain[1] > 0 && ke.push(Us(`${t}:grid:zero`, i ? F(0) : 1 - F(0), i ? "vertical" : "horizontal", void 0, "textMuted"))), ke.length > 0 && Ce.push(Ws(`${t}:grid`, ke));
	let je = [], Me = new Map(a.map((e) => [e.key, e])), Ne = ec(e, {
		annotations: n.annotations ?? [],
		positionOfX: te,
		bandRangeOfX: (e) => {
			if (v !== void 0) {
				let t = v.band(String(e));
				return t === void 0 ? void 0 : {
					start: t.start,
					end: t.end
				};
			}
			let t = te(e);
			return t === void 0 ? void 0 : {
				start: t,
				end: t
			};
		},
		valueToV: (e) => Number.isFinite(e) ? F(e) : void 0,
		toPoint: ne,
		toRect: re,
		seriesByKey: Me,
		pointOf: (e, t) => {
			let n = e.layers[0], r = n?.points[t];
			if (n === void 0 || r === void 0 || r.y === null) return;
			let i = te(r.x);
			if (i === void 0) return;
			let a = x.get(n)?.[g.indexOf(String(r.x))], o = F(a == null ? r.y : a.end);
			return {
				...ne(i, o),
				radius: n.pointRadius
			};
		},
		ids: je
	});
	Ce.push(...Ne.under);
	let Pe = o.filter((e) => e.mark === "bar").length, Fe = v !== void 0 && Pe > 1 && !f ? Xo(a.filter((e) => e.layers.some((e) => e.mark === "bar")).map((e) => e.id), [0, 1], .1) : void 0, Ie = Pe * g.length, Le = (e, t) => p === !0 ? !0 : p !== "auto" || t === "narrow" ? !1 : e <= (t === "wide" ? 12 : 8) && Math.max(Ie, e) <= (t === "wide" ? 24 : 12), Re = d === "band" ? "Category" : "X", ze = [], Be = me.title === void 0 ? "" : ` ${me.title}`;
	for (let e of a) {
		let n = [], r = [], a = [], o = [], s = [], c = [], l = e.interactive === "marks", u = e.points.filter((e) => e.y !== null).map((e) => e.y), m = u.length > 0 ? Math.min(...u) : void 0, h = u.length > 0 ? Math.max(...u) : void 0, _ = u.length === 0 ? `${e.label}: no data` : `${e.label}: ${Rs(u.length, "point")}, from ${k(m ?? 0)} to ${k(h ?? 0)}${Be}`, y = {
			role: "Series",
			title: e.label,
			summary: _,
			fields: [
				{
					label: "Series",
					value: e.label
				},
				{
					label: "Points",
					value: String(u.length)
				},
				...m === void 0 || h === void 0 ? [] : [{
					label: "Min",
					value: k(m)
				}, {
					label: "Max",
					value: k(h)
				}]
			]
		}, b = (t, n) => t.label ?? `${e.label} · ${String(t.x)}: ${n}`, S = (t, n, r) => ({
			role: t,
			title: `${e.label} · ${String(n.x)}`,
			fields: [
				{
					label: "Series",
					value: e.label
				},
				{
					label: Re,
					value: String(n.x)
				},
				{
					label: "Value",
					value: r
				}
			]
		}), C = (e, t, n, r) => ({
			inspect: S(n, e, t),
			...r ? {
				interactive: !0,
				label: b(e, t),
				...e.description === void 0 ? {} : { description: e.description }
			} : {}
		}), w = e.layers.some((e) => e.mark === "point"), T = e.layers.some((e) => e.mark === "line"), E = !1;
		for (let u of e.layers) {
			let m = u.spec.bind?.highlight === void 0 ? void 0 : { highlight: u.spec.bind.highlight }, h = u.mark === "bar" && d === "band" ? g.length : u.points.length, b = Q((e) => p !== !1 && Le(h, e)), S = As(Q((e) => !b[e])), D = Os(b);
			if (u.mark === "bar" && v !== void 0) {
				let a = x.get(u), s = /* @__PURE__ */ new Map();
				for (let e of u.points) s.set(String(e.x), e);
				g.forEach((c, d) => {
					let p = s.get(c);
					if (p === void 0 || p.y === null) return;
					let h = v.band(c);
					if (h === void 0) return;
					let g = a?.[d] ?? null, _ = g === null ? ie : g.start, y = g === null ? p.y : g.end, b = h.start, x = h.end, w = Fe?.band(e.id);
					w !== void 0 && (b = h.start + w.start * h.width, x = h.start + w.end * h.width);
					let T = re(b, x, F(_), F(y)), E = y < _, O = `${t}:bar:${e.id}:${d}`, A = k(p.y);
					if (n.push({
						id: O,
						type: "rect",
						position: {
							x: $(T.x),
							y: $(T.y),
							anchor: "top-left"
						},
						width: Ns(T.w),
						height: Ns(T.h),
						fill: p.tone ?? u.fill ?? u.tone,
						stroke: "none",
						radius: 2,
						...u.fillOpacity === void 0 ? {} : { opacity: u.fillOpacity },
						revealAnchor: i ? E ? "right" : "left" : E ? "top" : "bottom",
						...m === void 0 ? {} : { bind: m },
						...C(p, A, "Bar", l)
					}), r.push(O), D && !f) {
						let r = `${t}:label:${e.id}:${d}`;
						n.push($s(r, A, T, E, i, S)), o.push(r);
					}
				});
			} else if (u.mark === "bar") {
				let a = Math.max(.002, .6 / Math.max(1, u.points.length));
				u.points.forEach((s, c) => {
					if (s.y === null) return;
					let d = te(s.x);
					if (d === void 0) return;
					let f = re(d - a / 2, d + a / 2, F(ie), F(s.y)), p = s.y < ie, h = `${t}:bar:${e.id}:${c}`, g = k(s.y);
					if (n.push({
						id: h,
						type: "rect",
						position: {
							x: $(f.x),
							y: $(f.y),
							anchor: "top-left"
						},
						width: Ns(f.w),
						height: Ns(f.h),
						fill: s.tone ?? u.fill ?? u.tone,
						stroke: "none",
						radius: 2,
						...u.fillOpacity === void 0 ? {} : { opacity: u.fillOpacity },
						revealAnchor: i ? p ? "right" : "left" : p ? "top" : "bottom",
						...m === void 0 ? {} : { bind: m },
						...C(s, g, "Bar", l)
					}), r.push(h), D) {
						let r = `${t}:label:${e.id}:${c}`;
						n.push($s(r, g, f, p, i, S)), o.push(r);
					}
				});
			} else {
				let r = x.get(u), d = u.points.map((e, t) => {
					if (e.y === null) return null;
					let n = te(e.x);
					if (n === void 0) return null;
					let i = r?.[g.indexOf(String(e.x))] ?? null;
					return {
						index: t,
						point: e,
						uu: n,
						vv: F(i === null ? e.y : i.end),
						v0: F(i === null ? ie : i.start)
					};
				}), f = [], p = [];
				for (let e of d) e === null ? (p.length > 0 && f.push(p), p = []) : p.push(e);
				p.length > 0 && f.push(p);
				let h = u.spec.curve ?? "linear", v = u.spec.dash, b = u.mark === "line" || u.mark === "area" && !T;
				if (f.forEach((a, o) => {
					if (!(a.length < 2)) {
						if (u.mark === "area") {
							let s = o === 0 ? `${t}:area:${e.id}` : `${t}:area:${e.id}:${o}`, l = a.map((e) => ne(e.uu, e.vv)), d = a.map((e) => ne(e.uu, e.v0)).reverse(), f = i || r !== void 0;
							n.push({
								id: s,
								type: "polyline",
								position: {
									x: 0,
									y: 0
								},
								width: "100%",
								height: "100%",
								points: (f ? [...l, ...d] : l).map((e) => [$(e.x), $(e.y)]),
								...f ? { closed: !0 } : { baseline: $(1 - (a[0]?.v0 ?? 0)) },
								curve: h,
								fill: u.fill ?? u.tone,
								stroke: "none",
								opacity: u.fillOpacity ?? (u.fill === void 0 ? .25 : 1),
								revealAnchor: i ? "bottom" : "left",
								...m === void 0 ? {} : { bind: m }
							}), c.push(s);
						}
						if (b) {
							let r = o === 0 ? `${t}:line:${e.id}` : `${t}:line:${e.id}:${o}`, i = e.interactive === "series" && !E;
							n.push({
								id: r,
								type: "polyline",
								position: {
									x: 0,
									y: 0
								},
								width: "100%",
								height: "100%",
								points: a.map((e) => {
									let t = ne(e.uu, e.vv);
									return [$(t.x), $(t.y)];
								}),
								curve: h,
								stroke: u.tone,
								strokeWidth: 2,
								fill: "none",
								lineCap: "round",
								...v === void 0 ? {} : { dash: v },
								...m === void 0 ? {} : { bind: m },
								inspect: y,
								...i ? {
									interactive: !0,
									label: e.label,
									description: e.description ?? _
								} : {}
							}), s.push(r), i && (E = !0);
						}
					}
				}), u.mark === "point" || !w && (u.mark === "line" || !T)) {
					let r = /* @__PURE__ */ new Map();
					for (let e of f) for (let t of e) r.set(t, e.length);
					for (let i of d) {
						if (i === null) continue;
						let s = u.mark !== "point" && (r.get(i) ?? 1) < 2, c = u.mark === "point" ? Math.max(u.pointRadius, 1) : s ? Math.max(u.pointRadius, 2) : u.pointRadius;
						if (c <= 0) continue;
						let d = ne(i.uu, i.vv), f = `${t}:point:${e.id}:${i.index}`, p = k(i.point.y);
						if (n.push({
							id: f,
							type: "circle",
							position: {
								x: $(d.x),
								y: $(d.y),
								anchor: "center"
							},
							radius: c,
							fill: i.point.tone ?? u.tone,
							stroke: "none",
							...m === void 0 ? {} : { bind: m },
							...C(i.point, p, "Point", l)
						}), a.push(f), D) {
							let r = `${t}:label:${e.id}:${i.index}`;
							n.push($s(r, p, {
								x: d.x,
								y: d.y,
								w: 0,
								h: 0
							}, !1, !1, S, c + 2)), o.push(r);
						}
					}
				}
			}
		}
		let D = e.interactive === "series" && !E && n.length > 0, O = e.interactive !== "none" && n.length > 0, A = e.bind?.hidden === void 0 && e.bind?.opacity === void 0 ? void 0 : {
			...e.bind.hidden === void 0 ? {} : { hidden: e.bind.hidden },
			...e.bind.opacity === void 0 ? {} : { opacity: e.bind.opacity }
		}, j = {
			id: `${t}:series:${e.id}`,
			type: "group",
			layout: "coordinates",
			position: {
				x: 0,
				y: 0
			},
			width: "100%",
			height: "100%",
			...O ? { focusGroup: !0 } : {},
			label: e.label,
			description: e.description ?? _,
			inspect: y,
			...A === void 0 ? {} : { bind: A },
			...D ? { interactive: !0 } : {},
			...a.length > 0 ? { allowOverflow: !0 } : {},
			children: n
		};
		Ce.push(j);
		let M = [...r, ...a];
		Te.set(e.key, M), ze.push(...o), we[e.key] = {
			id: e.id,
			group: j.id,
			marks: M,
			bars: r,
			dots: a,
			labels: o,
			...s.length === 0 ? {} : {
				line: s[0] ?? "",
				lines: s
			},
			...c.length === 0 ? {} : {
				area: c[0] ?? "",
				areas: c
			}
		};
	}
	if (f && p !== !1 && v !== void 0 && Pe > 0) {
		let e = Q((e) => Le(g.length, e));
		if (Os(e)) {
			let n = As(Q((t) => !e[t])), r = [], a = o.filter((e) => e.mark === "bar");
			g.forEach((e, o) => {
				let s = v.band(e);
				if (s === void 0) return;
				let c = 0, l = 0, u = 0;
				for (let e of a) {
					let t = x.get(e)?.[o];
					t != null && (u += 1, t.end >= t.start ? c = Math.max(c, t.end) : l = Math.min(l, t.end));
				}
				if (u === 0) return;
				let d = c === 0 && l < 0, f = d ? re(s.start, s.end, F(l), F(0)) : re(s.start, s.end, F(0), F(c)), p = `${t}:label:stack:${o}`;
				r.push($s(p, k(c + l), f, d, i, n)), ze.push(p);
			}), r.length > 0 && Ce.push(Ws(`${t}:labels`, r));
		}
	}
	Ce.push(...Ne.over), Os(L.shown) && Ce.push({
		id: `${he[L.channel]}:line`,
		type: "rect",
		position: {
			x: 0,
			y: 1,
			anchor: "bottom-left"
		},
		width: "100%",
		height: 1,
		fill: "border",
		stroke: "none",
		radius: 0,
		...js(Q((e) => !L.shown[e]))
	});
	let Ve = o.some((e) => e.points.some((e) => e.y !== null));
	Ve || (r.push({
		severity: "warning",
		code: "empty-data",
		message: a.length === 0 ? "plot has no series" : "plot has no data points"
	}), Ce.push({
		id: `${t}:empty`,
		type: "text",
		text: "No data",
		textStyle: "caption",
		align: "center",
		position: {
			x: .5,
			y: .5,
			anchor: "center"
		}
	}));
	let He = {
		id: `${t}:area`,
		type: "group",
		layout: "coordinates",
		width: "fill",
		height: ks(Se),
		padding: ks(Q((e) => [
			_e,
			ve,
			ye[e],
			be[e]
		])),
		children: Ce
	}, Ue = (t) => Q((n) => {
		let r = t.ticks[n], i = t.side === "bottom" ? xs[n] : e.heights[n], a = r.map((e) => t.side === "bottom" ? Is(e.text) : ys), o = t.side === "bottom" ? Ss[n] : Math.max(2, Math.floor(i / ys)), s = vt(t.labelEvery, n), c = s !== void 0 && s >= 1 ? Math.floor(s) : Zs(r.length, Math.max(0, ...a), i, o), l = /* @__PURE__ */ new Set();
		return r.forEach((e, t) => {
			t % c === 0 && l.add(String(e.value));
		}), l;
	}), We = (e) => {
		let t = /* @__PURE__ */ new Map();
		for (let n of ft) for (let r of e.ticks[n]) t.set(String(r.value), r);
		return [...t.values()];
	}, Ge;
	if (Os(I.shown)) {
		let e = Ue(I), n = We(I).map((n, r) => ({
			id: `${t}:tick:${I.channel}:${r}`,
			type: "text",
			text: n.text,
			textStyle: "caption",
			align: "end",
			position: {
				x: 1,
				y: $(1 - n.position),
				anchor: "right"
			},
			...js(Q((t) => !I.shown[t] || !e[t].has(String(n.value))))
		}));
		Ge = {
			id: he[I.channel],
			type: "group",
			layout: "coordinates",
			width: ks(xe),
			height: ks(Se),
			padding: ks(Q((e) => [
				_e,
				ps,
				ye[e],
				0
			])),
			allowOverflow: !0,
			...js(Q((e) => !I.shown[e])),
			children: n
		};
	}
	let Ke;
	if (Os(L.shown)) {
		let e = Ue(L), n = [];
		We(L).forEach((r, i) => {
			let a = js(Q((t) => !L.shown[t] || !e[t].has(String(r.value)))), o = Is(r.text) / 2, s = Q((e) => {
				let t = r.position * xs[e];
				return t + o - xs[e] > ve - 2 ? "top-right" : o - t > xe[e] + be[e] - 2 ? "top-left" : "top";
			}), c = `${t}:tick:${L.channel}:${i}`;
			n.push({
				id: c,
				type: "text",
				text: r.text,
				textStyle: "caption",
				position: ks(Q((e) => ({
					x: $(r.position),
					y: 0,
					anchor: s[e]
				}))),
				...a
			}), L.kind === "linear" && n.push({
				id: `${c}:mark`,
				type: "rect",
				position: {
					x: $(r.position),
					y: 0,
					anchor: "bottom"
				},
				width: 1,
				height: ps,
				fill: "border",
				stroke: "none",
				radius: 0,
				...a
			});
		}), L.title !== void 0 && n.push({
			id: `${he[L.channel]}:title`,
			type: "text",
			text: L.title,
			textStyle: "label",
			position: {
				x: .5,
				y: 1,
				anchor: "bottom"
			}
		}), Ke = {
			id: he[L.channel],
			type: "group",
			layout: "coordinates",
			width: "fill",
			height: 26 + (L.title === void 0 ? 0 : bs),
			padding: ks(Q((e) => [
				ps,
				ve,
				0,
				xe[e] + be[e]
			])),
			allowOverflow: !0,
			...js(Q((e) => !L.shown[e])),
			children: n
		};
	}
	let qe = new Set(o.map((e) => e.mark)), Je = e.minimal ? "Sparkline" : qe.size === 0 ? "Chart" : qe.size > 1 ? "Combined chart" : qe.has("bar") ? "Bar chart" : qe.has("line") ? "Line chart" : qe.has("area") ? "Area chart" : "Scatter chart", Ye = d === "band" ? `over ${Rs(g.length, "category", "categories")}` : `over x from ${P(ee[0])} to ${P(ee[1])}${pe.title === void 0 ? "" : ` ${pe.title}`}`, Xe = n.description ?? (Ve ? `${Je} of ${Rs(a.length, "series", "series")} ${Ye}; y from ${O(E.domain[0])} to ${O(E.domain[1])}${Be}.` : `${Je} with no data.`), Ze = [], Qe;
	n.title !== void 0 && !e.minimal && (Qe = `${t}:title`, Ze.push({
		id: Qe,
		type: "text",
		text: n.title,
		textStyle: "bodyStrong"
	}));
	let $e = a.map((e) => {
		let t = e.layers.find((e) => e.mark === "bar") ?? e.layers[0], n = t === void 0 || t.mark === "bar" || t.mark === "area" ? "square" : t.mark === "point" ? "circle" : t.spec.dash === "dashed" || t.spec.dash === "dotted" ? "dashed" : "line";
		return {
			id: e.id,
			label: e.label,
			swatch: e.tone,
			shape: n
		};
	}), et = !e.minimal && n.legend !== !1 && (a.length > 1 || n.legend !== void 0), tt = n.legend === !1 ? "top" : n.legend?.position ?? "top", nt = et && $e.length > 0 ? {
		id: `${t}:legend`,
		type: "legend",
		items: $e,
		direction: "row"
	} : void 0;
	nt !== void 0 && tt === "top" && Ze.push(nt), I.title !== void 0 && Os(I.shown) && Ze.push({
		id: `${he[I.channel]}:title`,
		type: "text",
		text: I.title,
		textStyle: "label",
		...js(Q((e) => !I.shown[e]))
	}), Ge === void 0 ? Ze.push(He) : Ze.push({
		id: `${t}:body`,
		type: "group",
		layout: "row",
		gap: 0,
		width: "fill",
		children: [Ge, He]
	}), Ke !== void 0 && Ze.push(Ke), nt !== void 0 && tt === "bottom" && Ze.push(nt);
	let rt = {
		id: t,
		type: "group",
		layout: "stack",
		gap: e.minimal ? 0 : 8,
		width: "fill",
		label: n.title ?? Je,
		description: Xe,
		children: Ze
	}, it = tc(e, a, we, ze), at = {
		root: t,
		area: He.id,
		series: we,
		axes: {
			...Os(pe.shown) ? { x: he.x } : {},
			...Os(me.shown) ? { y: he.y } : {}
		},
		...nt === void 0 ? {} : { legend: nt.id },
		...Qe === void 0 ? {} : { title: Qe },
		...ke.length > 0 ? { grid: `${t}:grid` } : {},
		annotations: je
	};
	return {
		fragment: {
			nodes: [rt],
			tracks: it,
			summary: Xe,
			diagnostics: [...r]
		},
		handles: at,
		domains: {
			x: ee,
			y: E.domain
		},
		ticks: {
			x: d === "band" ? g : M?.ticks.wide ?? [],
			y: E.ticks.wide
		},
		description: Xe,
		diagnostics: r,
		markIds: Te
	};
}
function $s(e, t, n, r, i, a, o = 0) {
	let s = {
		id: o > 0 || i ? `${e}:text` : e,
		type: "text",
		text: t,
		textStyle: "caption",
		align: "center"
	};
	if (i) {
		let t = r ? "right" : "left";
		return {
			id: e,
			type: "group",
			layout: "stack",
			padding: r ? [
				0,
				4,
				0,
				0
			] : [
				0,
				0,
				0,
				4
			],
			position: {
				x: $(r ? n.x : n.x + n.w),
				y: $(n.y + n.h / 2),
				anchor: t
			},
			...a === void 0 ? {} : { hidden: a },
			children: [s]
		};
	}
	let c = r ? "top" : "bottom", l = n.x + n.w / 2, u = r ? n.y + n.h : n.y;
	return o > 0 ? {
		id: e,
		type: "group",
		layout: "stack",
		padding: r ? [
			o,
			0,
			0,
			0
		] : [
			0,
			0,
			o,
			0
		],
		position: {
			x: $(l),
			y: $(u),
			anchor: c
		},
		...a === void 0 ? {} : { hidden: a },
		children: [s]
	} : {
		...s,
		position: {
			x: $(l),
			y: $(u),
			anchor: c
		},
		...a === void 0 ? {} : { hidden: a }
	};
}
function ec(e, t) {
	let { p: n, diagnostics: r, horizontal: i } = e, a = [], o = [];
	return t.annotations.forEach((e, s) => {
		let c = `${n}:annotation:${s}`, l = (e) => {
			r.push({
				severity: "warning",
				code: "annotation-skipped",
				message: `annotation ${s}: ${e}`
			});
		};
		switch (e.type) {
			case "reference-line": {
				let n = e.tone ?? "textMuted", r, s;
				if (e.axis === "y") {
					if (typeof e.value != "number") return l("reference-line on the y axis needs a numeric value");
					let n = t.valueToV(e.value);
					if (n === void 0) return l("reference-line value is not finite");
					r = t.toPoint(0, ss(n, 0, 1)), s = t.toPoint(1, ss(n, 0, 1));
				} else {
					let n = t.positionOfX(e.value);
					if (n === void 0) return l(`reference-line x value ${String(e.value)} is not on the x scale`);
					r = t.toPoint(n, 0), s = t.toPoint(n, 1);
				}
				if (a.push({
					id: c,
					type: "polyline",
					position: {
						x: 0,
						y: 0
					},
					width: "100%",
					height: "100%",
					points: [[$(r.x), $(r.y)], [$(s.x), $(s.y)]],
					stroke: n,
					strokeWidth: 1,
					fill: "none",
					dash: e.dash ?? "dashed",
					...e.label === void 0 ? {} : {
						label: e.label,
						description: `Reference line at ${String(e.value)}`
					}
				}), e.label !== void 0) {
					let t = e.axis === "y" !== i;
					o.push({
						id: `${c}:label`,
						type: "text",
						text: e.label,
						textStyle: "caption",
						color: n,
						position: t ? {
							x: 1,
							y: $(Math.min(r.y, s.y)),
							anchor: "bottom-right"
						} : {
							x: $(Math.min(r.x, s.x)),
							y: 0,
							anchor: "top-left"
						}
					});
				}
				t.ids.push(c);
				break;
			}
			case "reference-band": {
				let n;
				if (e.axis === "y") {
					if (typeof e.from != "number" || typeof e.to != "number") return l("reference-band on the y axis needs numeric bounds");
					let r = t.valueToV(e.from), i = t.valueToV(e.to);
					if (r === void 0 || i === void 0) return l("reference-band bounds are not finite");
					n = t.toRect(0, 1, ss(r, 0, 1), ss(i, 0, 1));
				} else {
					let r = t.bandRangeOfX(e.from), i = t.bandRangeOfX(e.to);
					if (r === void 0 || i === void 0) return l("reference-band x bounds are not on the x scale");
					n = t.toRect(Math.min(r.start, i.start), Math.max(r.end, i.end), 0, 1);
				}
				a.push({
					id: c,
					type: "rect",
					position: {
						x: $(n.x),
						y: $(n.y),
						anchor: "top-left"
					},
					width: Ns(n.w),
					height: Ns(n.h),
					fill: e.tone ?? "surfaceMuted",
					stroke: "none",
					radius: 0,
					opacity: e.tone === void 0 ? .6 : .16,
					...e.label === void 0 ? {} : { label: e.label }
				}), e.label !== void 0 && o.push({
					id: `${c}:label`,
					type: "text",
					text: e.label,
					textStyle: "caption",
					color: e.tone ?? "textMuted",
					position: {
						x: $(n.x + n.w),
						y: $(n.y),
						anchor: "top-right"
					}
				}), t.ids.push(c);
				break;
			}
			case "point-label": {
				let n = e.series === void 0 ? [...t.seriesByKey.values()][0] : t.seriesByKey.get(e.series);
				if (n === void 0) return l(`unknown series ${String(e.series)}`);
				let r = t.pointOf(n, e.index);
				if (r === void 0) return l(`series ${n.key} has no datum at index ${e.index}`);
				let i = e.placement ?? "above", a = i === "above" ? "bottom" : i === "below" ? "top" : i === "left" ? "right" : "left", s = r.radius + 3, u = i === "above" ? [
					0,
					0,
					s,
					0
				] : i === "below" ? [
					s,
					0,
					0,
					0
				] : i === "left" ? [
					0,
					s,
					0,
					0
				] : [
					0,
					0,
					0,
					s
				];
				o.push({
					id: c,
					type: "group",
					layout: "stack",
					padding: u,
					position: {
						x: $(r.x),
						y: $(r.y),
						anchor: a
					},
					children: [{
						id: `${c}:text`,
						type: "text",
						text: e.text,
						textStyle: "caption",
						align: i === "left" ? "end" : i === "right" ? "start" : "center",
						...e.tone === void 0 ? {} : { color: e.tone }
					}]
				}), t.ids.push(c);
				break;
			}
			case "callout": {
				let n = t.positionOfX(e.x), r = t.valueToV(e.y);
				if (n === void 0 || r === void 0) return l("callout position is not on the scales");
				let i = t.toPoint(n, ss(r, 0, 1)), a = e.pointer ?? "up", s = Math.max(48, Math.min(e.maxWidth ?? 220, Math.ceil(Is(e.text) + 32))), u = {
					id: `${c}:callout`,
					type: "callout",
					text: e.text,
					pointer: a,
					width: s,
					maxLines: 6,
					...e.tone === void 0 ? {} : { tone: e.tone }
				}, d = a === "up" || a === "down" ? Math.max(0, s - 48) : 0, f = a === "up" ? "top" : a === "down" ? "bottom" : a === "left" ? "left" : a === "right" ? "right" : "top-left", p = a === "up" ? [
					4,
					0,
					0,
					d
				] : a === "down" ? [
					0,
					0,
					4,
					d
				] : a === "left" ? [
					0,
					0,
					0,
					4
				] : a === "right" ? [
					0,
					4,
					0,
					0
				] : [
					0,
					0,
					0,
					0
				];
				o.push({
					id: c,
					type: "group",
					layout: "stack",
					padding: p,
					position: {
						x: $(i.x),
						y: $(i.y),
						anchor: f
					},
					children: [u]
				}), t.ids.push(c);
				break;
			}
		}
	}), {
		under: a.length === 0 ? [] : [Ws(`${n}:annotations:under`, a, !0)],
		over: o.length === 0 ? [] : [Ws(`${n}:annotations:over`, o, !0)]
	};
}
function tc(e, t, n, r) {
	if (e.motion === "none") return [];
	let i = e.duration, a = [], o = e.horizontal ? "revealX" : "revealY";
	for (let r of t) {
		let t = n[r.key];
		if (t === void 0) continue;
		if (t.bars.length > 0) {
			if (t.bars.length > Cs) a.push(Vs(t.group, o, Bs(0, i * .8, 0, 1, e.easing)));
			else {
				let n = zs(t.bars.length, i * .4, 40), r = (i - n * (t.bars.length - 1)) * .85;
				t.bars.forEach((t, i) => {
					let s = n * i;
					a.push(Vs(t, o, Bs(s, s + r, 0, 1, e.easing)));
				});
			}
		}
		let s = i * .75;
		for (let n of t.lines ?? []) a.push(Vs(n, "progress", Bs(0, s, 0, 1, e.easing)));
		for (let n of t.areas ?? []) a.push(Vs(n, e.horizontal ? "revealY" : "revealX", Bs(0, s, 0, 1, e.easing)));
		let c = t.dots;
		if (c.length > 0) {
			let n = (t.lines?.length ?? 0) > 0 || (t.areas?.length ?? 0) > 0;
			if (c.length > Cs) a.push(Vs(t.group, "opacity", Bs(0, i * .8, n ? .4 : 0, 1, e.easing)));
			else if (n) {
				let t = zs(c.length, i * .3, 40);
				c.forEach((n, r) => {
					let o = i * .6 + t * r;
					a.push(Vs(n, "opacity", Bs(o, Math.min(i, o + 160), 0, 1, e.easing)));
				});
			} else {
				let t = zs(c.length, i * .5, 40), n = Math.max(120, (i - t * (c.length - 1)) * .6);
				c.forEach((r, o) => {
					let s = t * o, c = Math.min(i, s + n);
					a.push(Vs(r, "opacity", Bs(s, c, 0, 1, e.easing))), a.push(Vs(r, "scale", Bs(s, c, .6, 1, e.easing)));
				});
			}
		}
	}
	for (let t of r) a.push(Vs(t, "opacity", Bs(i * .7, i, 0, 1, e.easing)));
	return a;
}
function nc(e, t) {
	let { p: n, spec: r, diagnostics: i } = e, a = t.rows.map(String), o = t.columns.map(String), s = a.map((e, n) => o.map((e, r) => {
		let i = t.values[n]?.[r];
		return typeof i == "number" && Number.isFinite(i) ? i : null;
	}));
	(t.values.length !== a.length || t.values.some((e) => e.length !== o.length)) && i.push({
		severity: "warning",
		code: "ragged-heatmap",
		message: "heatmap values do not match rows × columns; missing cells are empty and extras are ignored"
	});
	let c = s.flat().filter((e) => e !== null), l = t.negativeTone !== void 0, u = t.domain === void 0 || t.domain === "auto" ? void 0 : t.domain, d = u?.[0] ?? (c.length > 0 ? Math.min(...c) : 0), f = u?.[1] ?? (c.length > 0 ? Math.max(...c) : 1);
	d === f && (d === 0 ? f = 1 : (d = Math.min(d, 0), f = Math.max(f, 0), d === f && (f = d + 1)));
	let p = Math.max(Math.abs(d), Math.abs(f)) || 1, m = t.format ?? {}, h = (e) => is(e, m), g = t.tone ?? (l ? "chartPositive" : "chart1"), _ = t.negativeTone ?? "chartNegative", v = (e) => {
		let t = Math.round(ss(e, 0, 1) * 7) / 7;
		return Math.round((Ts + t * .85) * 1e3) / 1e3;
	}, y = (e) => l ? {
		fill: e < 0 ? _ : g,
		opacity: v(Math.abs(e) / p)
	} : {
		fill: g,
		opacity: v((e - d) / (f - d))
	}, b = Xo(a, [0, 1], .08), x = Xo(o, [0, 1], .08), S = a.length * o.length, C = S <= 144, w = e.minimal ? void 0 : t.cellLabels, T = [], E = [], D = [], O = [];
	a.forEach((e, r) => {
		let i = [], a = b.band(r);
		o.forEach((o, c) => {
			let l = x.band(c);
			if (a === void 0 || l === void 0) return;
			let u = s[r]?.[c] ?? null, d = `${n}:cell:${r}:${c}`, f = u === null ? {
				fill: "surfaceMuted",
				opacity: 1
			} : y(u), p = u === null ? "–" : h(u);
			if (E.push({
				id: d,
				type: "rect",
				position: {
					x: $(l.start),
					y: $(a.start),
					anchor: "top-left"
				},
				width: Ns(l.width),
				height: Ns(a.width),
				fill: f.fill,
				stroke: "none",
				radius: 2,
				...f.opacity === 1 ? {} : { opacity: f.opacity },
				inspect: {
					role: "Cell",
					title: `${e} · ${o}`,
					fields: [
						{
							label: t.rowLabel ?? "Row",
							value: e
						},
						{
							label: t.columnLabel ?? "Column",
							value: o
						},
						{
							label: "Value",
							value: p
						}
					]
				},
				...C ? {
					interactive: !0,
					label: `${e} · ${o}: ${p}`
				} : {}
			}), i.push(d), D.push(d), w !== void 0 && u !== null) {
				let e = As(Q((e) => !Ms(w, e, !1)));
				if (e !== !0) {
					let t = `${d}:label`;
					E.push({
						id: t,
						type: "text",
						text: p,
						textStyle: "caption",
						align: "center",
						position: {
							x: $(l.center),
							y: $(a.center),
							anchor: "center"
						},
						...e === void 0 ? {} : { hidden: e }
					}), O.push(t);
				}
			}
		}), T.push(i);
	});
	let k = `values from ${h(d)} to ${h(f)}`, A = {
		id: `${n}:series:heatmap`,
		type: "group",
		layout: "coordinates",
		position: {
			x: 0,
			y: 0
		},
		width: "100%",
		height: "100%",
		focusGroup: !0,
		label: r.title ?? "Heatmap",
		description: `${Rs(a.length, "row")} by ${Rs(o.length, "column")}, ${k}`,
		inspect: {
			role: "Series",
			title: r.title ?? "Heatmap",
			fields: [
				{
					label: "Rows",
					value: String(a.length)
				},
				{
					label: "Columns",
					value: String(o.length)
				},
				{
					label: "Min",
					value: h(d)
				},
				{
					label: "Max",
					value: h(f)
				}
			]
		},
		...C ? {} : { interactive: !0 },
		children: E
	}, j = o.map((e, t) => ({
		position: x.band(t)?.center ?? 0,
		text: e,
		value: e
	})), M = a.map((e, t) => ({
		position: b.band(t)?.center ?? 0,
		text: e,
		value: e
	})), N = r.axes ?? {}, P = e.minimal ? !1 : N.x ?? {}, ee = e.minimal ? !1 : N.y ?? {}, te = Q((e) => P !== !1 && !Ms(P.hidden, e, !1)), F = Q((e) => ee !== !1 && !Ms(ee.hidden, e, !1)), ne = Q((e) => {
		if (!F[e]) return 0;
		let t = Math.max(0, ...M.map((e) => Is(e.text)));
		return Math.ceil(ss(t + ps + 4, gs, _s[e]));
	}), re = (P === !1 ? void 0 : P.label) ?? t.columnLabel, ie = (ee === !1 ? void 0 : ee.label) ?? t.rowLabel, ae = e.minimal ? 3 : ms, oe = e.minimal ? ae : hs, se = Q((e) => te[e] ? 0 : ae), ce = Q((e) => F[e] ? 0 : ae), le = Q((t) => e.heights[t] + ae + se[t]), ue = {
		x: `${n}:axis:x`,
		y: `${n}:axis:y`
	}, de = [A];
	S === 0 && (i.push({
		severity: "warning",
		code: "empty-data",
		message: "heatmap has no cells"
	}), de.push({
		id: `${n}:empty`,
		type: "text",
		text: "No data",
		textStyle: "caption",
		align: "center",
		position: {
			x: .5,
			y: .5,
			anchor: "center"
		}
	}));
	let fe = {
		id: `${n}:area`,
		type: "group",
		layout: "coordinates",
		width: "fill",
		height: ks(le),
		padding: ks(Q((e) => [
			ae,
			oe,
			se[e],
			ce[e]
		])),
		children: de
	}, pe;
	if (Os(F)) {
		let t = Q((t) => {
			let n = ee === !1 ? void 0 : vt(ee.labelEvery, t);
			return n !== void 0 && n >= 1 ? Math.floor(n) : Zs(a.length, ys, e.heights[t], Math.max(2, Math.floor(e.heights[t] / ys)));
		});
		pe = {
			id: ue.y,
			type: "group",
			layout: "coordinates",
			width: ks(ne),
			height: ks(le),
			padding: ks(Q((e) => [
				ae,
				ps,
				se[e],
				0
			])),
			allowOverflow: !0,
			...js(Q((e) => !F[e])),
			children: M.map((e, r) => ({
				id: `${n}:tick:y:${r}`,
				type: "text",
				text: e.text,
				textStyle: "caption",
				align: "end",
				position: {
					x: 1,
					y: $(e.position),
					anchor: "right"
				},
				...js(Q((e) => !F[e] || r % t[e] !== 0))
			}))
		};
	}
	let me;
	if (Os(te)) {
		let e = Q((e) => {
			let t = P === !1 ? void 0 : vt(P.labelEvery, e);
			return t !== void 0 && t >= 1 ? Math.floor(t) : Zs(o.length, Math.max(0, ...j.map((e) => Is(e.text))), xs[e], Ss[e]);
		}), t = j.map((t, r) => ({
			id: `${n}:tick:x:${r}`,
			type: "text",
			text: t.text,
			textStyle: "caption",
			position: {
				x: $(t.position),
				y: 0,
				anchor: "top"
			},
			...js(Q((t) => !te[t] || r % e[t] !== 0))
		}));
		re !== void 0 && t.push({
			id: `${ue.x}:title`,
			type: "text",
			text: re,
			textStyle: "label",
			position: {
				x: .5,
				y: 1,
				anchor: "bottom"
			}
		}), me = {
			id: ue.x,
			type: "group",
			layout: "coordinates",
			width: "fill",
			height: 26 + (re === void 0 ? 0 : bs),
			padding: ks(Q((e) => [
				ps,
				oe,
				0,
				ne[e] + ce[e]
			])),
			allowOverflow: !0,
			...js(Q((e) => !te[e])),
			children: t
		};
	}
	let I = [], L;
	r.title !== void 0 && !e.minimal && (L = `${n}:title`, I.push({
		id: L,
		type: "text",
		text: r.title,
		textStyle: "bodyStrong"
	}));
	let he;
	if (!e.minimal && r.legend !== !1 && S > 0) {
		let e = (e, t, n) => ({
			id: e,
			type: "rect",
			width: 14,
			height: 10,
			fill: t,
			stroke: "none",
			radius: 1,
			...n === 1 ? {} : { opacity: n }
		}), t = [];
		if (l) {
			for (let r = 7; r >= 0; --r) t.push(e(`${n}:legend:neg:${r}`, _, v(r / 7)));
			for (let r = 0; r < ws; r += 1) t.push(e(`${n}:legend:pos:${r}`, g, v(r / 7)));
		} else for (let r = 0; r < ws; r += 1) t.push(e(`${n}:legend:step:${r}`, g, v(r / 7)));
		he = {
			id: `${n}:legend`,
			type: "group",
			layout: "row",
			gap: 6,
			align: "center",
			children: [
				{
					id: `${n}:legend:min`,
					type: "text",
					text: h(l ? -p : d),
					textStyle: "caption"
				},
				{
					id: `${n}:legend:ramp`,
					type: "group",
					layout: "row",
					gap: 2,
					align: "center",
					children: t
				},
				{
					id: `${n}:legend:max`,
					type: "text",
					text: h(l ? p : f),
					textStyle: "caption"
				}
			]
		};
	}
	let ge = r.legend === !1 ? "top" : r.legend?.position ?? "top";
	he !== void 0 && ge === "top" && I.push(he), ie !== void 0 && Os(F) && I.push({
		id: `${ue.y}:title`,
		type: "text",
		text: ie,
		textStyle: "label",
		...js(Q((e) => !F[e]))
	}), pe === void 0 ? I.push(fe) : I.push({
		id: `${n}:body`,
		type: "group",
		layout: "row",
		gap: 0,
		width: "fill",
		children: [pe, fe]
	}), me !== void 0 && I.push(me), he !== void 0 && ge === "bottom" && I.push(he);
	let _e = r.description ?? (S > 0 ? `Heatmap of ${Rs(a.length, "row")} by ${Rs(o.length, "column")}; ${k}.` : "Heatmap with no data."), ve = {
		id: n,
		type: "group",
		layout: "stack",
		gap: e.minimal ? 0 : 8,
		width: "fill",
		label: r.title ?? "Heatmap",
		description: _e,
		children: I
	}, ye = [];
	if (e.motion !== "none" && S > 0) {
		let t = e.duration;
		if (S > Cs) ye.push(Vs(A.id, "opacity", Bs(0, t * .8, 0, 1, e.easing)));
		else {
			let n = zs(a.length, t * .5, 120), r = (t - n * Math.max(0, a.length - 1)) * .7;
			T.forEach((t, i) => {
				let a = n * i;
				for (let n of t) ye.push(Vs(n, "opacity", Bs(a, a + r, 0, 1, e.easing)));
			});
		}
		for (let n of O) ye.push(Vs(n, "opacity", Bs(t * .7, t, 0, 1, e.easing)));
	}
	let be = {
		id: "heatmap",
		group: A.id,
		marks: D,
		bars: [],
		dots: [],
		labels: O
	}, xe = {
		root: n,
		area: fe.id,
		series: { heatmap: be },
		axes: {
			...Os(te) ? { x: ue.x } : {},
			...Os(F) ? { y: ue.y } : {}
		},
		...he === void 0 ? {} : { legend: he.id },
		...L === void 0 ? {} : { title: L },
		annotations: [],
		cells: T
	};
	return {
		fragment: {
			nodes: [ve],
			tracks: ye,
			summary: _e,
			diagnostics: [...i]
		},
		handles: xe,
		domains: {
			x: o,
			y: a
		},
		ticks: {
			x: o,
			y: a
		},
		description: _e,
		diagnostics: i,
		markIds: /* @__PURE__ */ new Map([["heatmap", D]])
	};
}
//#endregion
//#region ../plot/dist/marks.js
function rc(e, t = {}) {
	return {
		kind: e,
		...t
	};
}
function ic(e = {}) {
	return rc("bar", e);
}
function ac(e = {}) {
	return rc("grouped-bar", e);
}
function oc(e = {}) {
	return rc("stacked-bar", e);
}
function sc(e = {}) {
	return rc("line", e);
}
function cc(e = {}) {
	return rc("area", e);
}
function lc(e = {}) {
	return rc("dot", e);
}
function uc(e = {}) {
	return rc("sparkline", e);
}
function dc(e) {
	return {
		kind: "heatmap",
		...e
	};
}
function fc(e) {
	let t = e.y === void 0 ? "x" : "y";
	return {
		type: "reference-line",
		axis: t,
		value: t === "y" ? e.y ?? 0 : e.x ?? 0,
		...e.label === void 0 ? {} : { label: e.label },
		...e.tone === void 0 ? {} : { tone: e.tone },
		...e.dash === void 0 ? {} : { dash: e.dash }
	};
}
function pc(e) {
	let t = e.y === void 0 ? "x" : "y", [n, r] = t === "y" ? e.y ?? [0, 0] : e.x ?? [0, 0];
	return {
		type: "reference-band",
		axis: t,
		from: n,
		to: r,
		...e.label === void 0 ? {} : { label: e.label },
		...e.tone === void 0 ? {} : { tone: e.tone }
	};
}
function mc(e) {
	return {
		type: "callout",
		...e
	};
}
function hc(e) {
	return {
		type: "point-label",
		...e
	};
}
//#endregion
//#region ../plot/dist/index.js
function gc(e, t = {}) {
	if (!Array.isArray(e)) return Gs(e, t);
	let n = Jo(e, t), r = Gs(n.spec, t), i = {}, a = /* @__PURE__ */ new Map();
	for (let e of n.seriesKeys) {
		let t = r.handles.series[e.id];
		t !== void 0 && (i[e.key] = t);
		let n = r.markIds.get(e.id);
		n !== void 0 && a.set(e.key, n);
	}
	let o = {
		...r,
		handles: {
			...r.handles,
			series: i
		},
		markIds: a
	};
	if (n.diagnostics.length === 0) return o;
	let s = [...n.diagnostics, ...r.diagnostics];
	return {
		...o,
		fragment: {
			...o.fragment,
			diagnostics: [...n.diagnostics, ...o.fragment.diagnostics ?? []]
		},
		diagnostics: s
	};
}
var _c = {
	slug: "benchmark-breakdown",
	order: 90,
	title: "Benchmark comparison",
	summary: "Grouped throughput and stacked cost views explain the same illustrative benchmark.",
	concept: "Quantitative comparison with provenance and cost decomposition.",
	interaction: "Inspect bars and series to read exact values and their role in the comparison.",
	animation: "Both views rise together, aligning the headline result with its runtime breakdown.",
	source: "Kineglyph quantitative example; all values are illustrative.",
	scene: br("benchmark-breakdown", {
		title: "Benchmark results need a comparison and an explanation",
		description: "Illustrative grouped throughput and stacked runtime breakdowns compare scalar and bulk schematic writes.",
		metadata: {
			data: "illustrative",
			family: "quantitative"
		}
	}, (e) => {
		let t = e.heading("One benchmark, two useful questions"), n = gc([
			{
				workload: "Dense box",
				scalar: .7,
				bulk: 18.4
			},
			{
				workload: "Sparse points",
				scalar: .6,
				bulk: 9.8
			},
			{
				workload: "Mixed ids",
				scalar: .5,
				bulk: 6.2
			}
		], {
			id: "benchmark-grouped",
			x: "workload",
			y: ["scalar", "bulk"],
			marks: ac(),
			title: "Illustrative throughput",
			description: "Millions of cells per second for scalar and bulk schematic writes; illustrative values only.",
			axes: {
				x: { label: "Workload" },
				y: {
					label: "Throughput (M cells/s)",
					format: { digits: 1 }
				}
			},
			grid: "y",
			legend: { position: "top" },
			valueLabels: !1,
			height: {
				wide: 218,
				compact: 190,
				narrow: 170
			},
			motion: "auto",
			duration: 900
		}), r = gc([
			{
				method: "Scalar loop",
				binding: 38,
				parsing: 34,
				writing: 28
			},
			{
				method: "set_blocks",
				binding: 8,
				parsing: 12,
				writing: 25
			},
			{
				method: "fill_cuboid",
				binding: 4,
				parsing: 3,
				writing: 14
			}
		], {
			id: "benchmark-stacked",
			x: "method",
			y: [
				"binding",
				"parsing",
				"writing"
			],
			marks: oc(),
			title: "Illustrative time per batch",
			description: "Stacked milliseconds split into binding, parsing, and writing costs for three methods; illustrative values only.",
			axes: {
				x: { label: "Method" },
				y: {
					label: "Time (ms)",
					format: { digits: 0 }
				}
			},
			grid: "y",
			legend: { position: "top" },
			valueLabels: !1,
			height: {
				wide: 218,
				compact: 190,
				narrow: 170
			},
			motion: "auto",
			duration: 900
		}), i = e.add(n), a = e.add(r), o = e.flow([i, a], {
			gap: {
				wide: 24,
				compact: 20,
				narrow: 18
			},
			align: "stretch",
			width: "fill"
		}), s = e.caption("Illustrative data—not a published Nucleation benchmark. The paired views show why a headline number needs a cost breakdown.");
		e.stack([
			t,
			o,
			s
		], {
			gap: {
				wide: 18,
				compact: 16,
				narrow: 14
			},
			width: "fill"
		}), e.sequence([
			e.reveal(t, { offset: 8 }),
			[e.reveal(i), e.reveal(a)],
			e.reveal(s, { offset: 6 })
		]);
	})
}, vc = [
	"js",
	"python",
	"kotlin",
	"php",
	"c",
	"cpp"
], yc = [...vc, "rust"], bc = {
	js: {
		title: "JavaScript / TypeScript",
		runtime: "WASM",
		control: "JS / TS",
		motif: "world",
		snippet: "import { Schematic } from \"nucleation\"",
		detail: "WASM keeps the same byte and JSON contracts in browsers and Node; nothing native to build or ship."
	},
	python: {
		title: "Python",
		runtime: "nanobind native module",
		control: "Python",
		motif: "terminal",
		snippet: "import nucleation as nc",
		detail: "A nanobind native module: Python objects wrap the same byte contract with no copies in between."
	},
	kotlin: {
		title: "Kotlin / JVM",
		runtime: "JNA",
		control: "Kotlin / JVM",
		motif: "cube",
		snippet: "import dev.nucleation.Schematic",
		detail: "JNA loads the shared library on the JVM and calls the generated symbol names directly."
	},
	php: {
		title: "PHP",
		runtime: "FFI",
		control: "PHP",
		motif: "plug",
		snippet: "use Nucleation\\Schematic;",
		detail: "PHP FFI binds the C ABI at runtime; names and payload shapes are generated, never hand-written."
	},
	c: {
		title: "C",
		runtime: "stable ABI headers",
		control: "C",
		motif: "file",
		snippet: "#include \"nucleation.h\"",
		detail: "Stable ABI headers are the contract every other binding is built on; the symbols never drift."
	},
	cpp: {
		title: "C++",
		runtime: "typed C ABI wrappers",
		control: "C++",
		motif: "layers",
		snippet: "#include <nucleation.hpp>",
		detail: "Typed wrappers add RAII and real types over the same exported C symbols, so nothing is duplicated."
	},
	rust: {
		title: "Rust",
		runtime: "native crate · direct",
		control: "Rust",
		motif: "rust",
		snippet: "use nucleation::Schematic;",
		detail: "Native Rust skips the bridge entirely: crates depend on the core and call its API directly."
	}
}, xc = "One definition, seven surfaces", Sc = "src/bridge/*.rs", Cc = "Annotate the core once in src/bridge; Diplomat generates naming, byte and JSON contracts for every surface.", wc = {
	...Object.fromEntries(yc.map((e) => [`FOCUS_${e.toUpperCase()}`, e])),
	RESET: "overview"
};
function Tc(e) {
	return {
		label: bc[e].title,
		entry: [{
			type: "set",
			var: "surface",
			value: e
		}, {
			type: "select",
			node: `surface-${e}`
		}],
		on: wc
	};
}
function Ec(e) {
	return e.charAt(0).toUpperCase() + e.slice(1);
}
function Dc() {
	let e = {};
	for (let t of yc) e[`${t}Focus`] = {
		when: {
			var: "surface",
			op: "eq",
			value: t
		},
		then: 1,
		else: 0
	}, e[`${t}Dim`] = {
		when: {
			var: "surface",
			op: "in",
			value: ["none", t]
		},
		then: 1,
		else: .55
	}, e[`edge${Ec(t)}`] = {
		when: {
			var: "surface",
			op: "eq",
			value: t
		},
		then: 1,
		else: 0
	}, e[`edge${Ec(t)}Tone`] = {
		when: {
			var: "surface",
			op: "in",
			value: ["none", t]
		},
		then: "neutral",
		else: "muted"
	};
	return e;
}
var Oc = {
	id: "binding-surfaces",
	initial: "overview",
	variables: { surface: "none" },
	states: {
		overview: {
			label: "All surfaces",
			entry: [{
				type: "set",
				var: "surface",
				value: "none"
			}, {
				type: "select",
				node: null
			}],
			on: wc
		},
		...Object.fromEntries(yc.map((e) => [e, Tc(e)]))
	},
	signals: {
		focusTitle: {
			match: { var: "surface" },
			cases: Object.fromEntries(yc.map((e) => [e, `${bc[e].title} · ${bc[e].runtime}`])),
			default: xc
		},
		snippet: {
			match: { var: "surface" },
			cases: Object.fromEntries(yc.map((e) => [e, bc[e].snippet])),
			default: Sc
		},
		detail: {
			match: { var: "surface" },
			cases: Object.fromEntries(yc.map((e) => [e, bc[e].detail])),
			default: Cc
		},
		coreFocus: {
			when: {
				var: "surface",
				op: "neq",
				value: "none"
			},
			then: 1,
			else: 0
		},
		bridgeFocus: {
			when: {
				var: "surface",
				op: "in",
				value: [...vc]
			},
			then: 1,
			else: 0
		},
		bridgeDim: {
			when: {
				var: "surface",
				op: "eq",
				value: "rust"
			},
			then: .55,
			else: 1
		},
		bridgeTone: {
			when: {
				var: "surface",
				op: "eq",
				value: "rust"
			},
			then: "muted",
			else: "neutral"
		},
		...Dc()
	}
};
function kc(e) {
	return {
		...e,
		children: e.children.map((t) => t.type === "group" && t.id === `${e.id}-header` ? {
			...t,
			width: "fill"
		} : t)
	};
}
function Ac(e, t, n, r) {
	return {
		...z(e, n, r),
		...t === void 0 ? {} : { layout: t }
	};
}
function jc(e, t) {
	return z(`col-${e}`, [kc(Xn(e, {
		eyebrow: t.eyebrow,
		title: t.title,
		body: t.body,
		motif: t.motif,
		tone: t.tone,
		bind: t.bind,
		compact: !0
	}))], {
		width: "fill",
		height: "fill",
		justify: "center",
		grow: t.grow
	});
}
var Mc = jc("core", {
	eyebrow: "Native surface",
	title: "Rust core",
	body: "Schematics, fields, and simulation live here once.",
	motif: "rust",
	tone: "accent",
	bind: { highlight: "coreFocus" },
	grow: 5
}), Nc = jc("annotations", {
	eyebrow: "src/bridge",
	title: "Annotations",
	body: "Attributes mark what may cross the boundary.",
	motif: "code",
	tone: "accent",
	bind: {
		highlight: "bridgeFocus",
		opacity: "bridgeDim"
	},
	grow: 6
}), Pc = jc("diplomat", {
	eyebrow: "Generator",
	title: "Diplomat",
	body: "Generated contracts: naming, bytes, JSON.",
	motif: "bridge",
	tone: "info",
	bind: {
		highlight: "bridgeFocus",
		opacity: "bridgeDim"
	},
	grow: 6
});
function Fc(e) {
	let t = bc[e];
	return z(`wrap-${e}`, [kc(Xn(`surface-${e}`, {
		eyebrow: t.runtime,
		title: t.title,
		motif: t.motif,
		tone: "success",
		interactive: !0,
		onActivate: `FOCUS_${e.toUpperCase()}`,
		description: t.detail,
		bind: {
			highlight: `${e}Focus`,
			opacity: `${e}Dim`
		},
		metadata: { surface: e },
		compact: !0,
		...e === "rust" ? { frame: {
			fill: "surface",
			stroke: "border",
			dash: "dashed"
		} } : {}
	}))], { width: "fill" });
}
var Ic = z("surfaces", [
	Bn("surfaces-eyebrow", "Six generated surfaces"),
	Ac("surfaces-grid", {
		wide: "stack",
		compact: "grid",
		narrow: "stack"
	}, vc.map(Fc), {
		gap: 10,
		width: "fill",
		columns: {
			compact: 2,
			narrow: 1
		}
	}),
	Bn("direct-eyebrow", "Direct, no bridge"),
	Ac("surfaces-direct", {
		wide: "stack",
		compact: "grid",
		narrow: "stack"
	}, [Fc("rust")], {
		gap: 10,
		width: "fill",
		columns: {
			compact: 2,
			narrow: 1
		}
	})
], {
	gap: 10,
	width: "fill",
	grow: 9
}), Lc = z("footer", [{
	id: "footer-legend",
	type: "legend",
	items: [
		{
			id: "one-definition",
			label: "one definition",
			swatch: "accent"
		},
		{
			id: "generated-naming",
			label: "generated naming",
			swatch: "info"
		},
		{
			id: "shared-contracts",
			label: "shared byte and JSON contracts",
			swatch: "success"
		}
	],
	gap: 18
}, Ac("footer-focus", {
	wide: "row",
	narrow: "stack"
}, [z("footer-copy", [Bn("footer-title", xc, { bind: { text: "focusTitle" } }), Un("footer-detail", Cc, {
	bind: { text: "detail" },
	maxLines: 3,
	width: "fill"
})], {
	gap: 3,
	width: "fill"
}), Wn("footer-snippet", Sc, {
	bind: { text: "snippet" },
	tone: "accent",
	maxLines: 1
})], {
	gap: {
		wide: 24,
		narrow: 8
	},
	align: {
		wide: "center",
		narrow: "start"
	},
	width: "fill"
})], {
	gap: 12,
	padding: [12, 16],
	frame: {
		fill: "surfaceMuted",
		stroke: "border",
		dash: "dashed"
	},
	width: "fill"
});
function Rc(e) {
	return {
		id: `gen-${e}`,
		from: {
			node: "diplomat",
			side: "right"
		},
		to: {
			node: `surface-${e}`,
			side: "left"
		},
		route: "curve",
		curvature: .2,
		head: "arrow",
		packets: {
			count: 1,
			period: 2e3
		},
		hidden: {
			wide: !1,
			compact: !0
		},
		description: `Diplomat generates the ${bc[e].title} surface`,
		bind: {
			highlight: `edge${Ec(e)}`,
			tone: `edge${Ec(e)}Tone`
		}
	};
}
var zc = {
	slug: "bindings-and-languages",
	order: 7,
	title: "Bindings and languages",
	summary: "One annotated Rust bridge and Diplomat generate six language surfaces that share naming, byte, and JSON contracts.",
	concept: "Bindings and languages: the Rust core and bridge annotations generate language surfaces with shared semantics.",
	interaction: "Pick a surface (click a card, keyboard, or the buttons) to light its path from the core, dim the others, and read its import line and guarantee; Rust shows the direct call that skips the bridge.",
	animation: "The core is annotated, Diplomat generates, six connectors fan out carrying packets to each language, native Rust bypasses the bridge, and the shared-contract strip appears.",
	source: "bindings-and-languages/binding-pipeline.svg",
	scene: Ct({
		schemaVersion: 2,
		id: "bindings-and-languages",
		title: "One annotated Rust bridge generating six foreign language bindings",
		description: "The Rust core is annotated once in src/bridge; Diplomat generates naming, byte and JSON contracts, and JavaScript, Python, Kotlin, PHP, C, and C++ surfaces all share them. Native Rust bypasses the bridge and calls the core directly.",
		breakpoints: {
			wide: 900,
			compact: 600
		},
		root: z("root", [Yn("pipeline", [
			Mc,
			Nc,
			Pc,
			Ic
		], {
			gap: {
				wide: 44,
				compact: 26
			},
			align: "stretch",
			width: "fill",
			padding: {
				wide: 0,
				compact: [0, 22]
			}
		}), Lc], {
			gap: 22,
			width: "fill"
		}),
		edges: [
			{
				id: "core-annotations",
				from: {
					node: "core",
					side: {
						wide: "right",
						compact: "bottom"
					}
				},
				to: {
					node: "annotations",
					side: {
						wide: "left",
						compact: "top"
					}
				},
				route: "straight",
				head: "arrow",
				packets: {
					count: 1,
					period: 1800
				},
				description: "The core is annotated in src/bridge",
				bind: {
					highlight: "bridgeFocus",
					tone: "bridgeTone"
				}
			},
			{
				id: "annotations-diplomat",
				from: {
					node: "annotations",
					side: {
						wide: "right",
						compact: "bottom"
					}
				},
				to: {
					node: "diplomat",
					side: {
						wide: "left",
						compact: "top"
					}
				},
				route: "straight",
				head: "arrow",
				packets: {
					count: 1,
					period: 1800
				},
				description: "Diplomat reads the annotations",
				bind: {
					highlight: "bridgeFocus",
					tone: "bridgeTone"
				}
			},
			...vc.map(Rc),
			{
				id: "gen-trunk",
				from: {
					node: "diplomat",
					side: "bottom"
				},
				to: {
					node: "surfaces-grid",
					side: "top"
				},
				route: "straight",
				head: "triangle",
				stroke: "flow",
				hidden: {
					wide: !0,
					compact: !1
				},
				description: "Diplomat generates all six surfaces",
				bind: {
					highlight: "bridgeFocus",
					tone: "bridgeTone"
				}
			},
			{
				id: "core-rust",
				from: {
					node: "core",
					side: {
						wide: "bottom",
						compact: "left"
					}
				},
				to: {
					node: "surface-rust",
					side: "left"
				},
				route: "orthogonal",
				cornerRadius: 10,
				head: "arrow",
				tail: "dot",
				stroke: "dashed",
				labels: [{
					text: "direct call, no bridge",
					placement: "middle",
					hidden: {
						wide: !1,
						compact: !0
					}
				}],
				description: "Native Rust calls the core directly, bypassing Diplomat",
				bind: {
					highlight: "edgeRust",
					tone: "edgeRustTone"
				}
			}
		],
		timeline: Ve([
			Pe("col-core", 0, 450, { scale: .96 }),
			Fe("core-annotations", 450, 850),
			Ie("core-annotations", 850),
			Pe("col-annotations", 650, 1050, { scale: .96 }),
			Fe("annotations-diplomat", 1050, 1450),
			Ie("annotations-diplomat", 1450),
			Pe("col-diplomat", 1250, 1650, { scale: .96 }),
			Re("diplomat-motif", 1650, 700),
			je("surfaces-eyebrow", 1750, 2150),
			Fe("gen-trunk", 1800, 2300),
			...vc.flatMap((e, t) => {
				let n = 1850 + t * 220;
				return [
					Fe(`gen-${e}`, n, n + 450),
					Ie(`gen-${e}`, n + 450),
					Pe(`wrap-${e}`, n + 200, n + 600, { offset: -8 })
				];
			}),
			je("direct-eyebrow", 3450, 3850),
			Fe("core-rust", 3450, 4100),
			Pe("wrap-rust", 3850, 4250, { offset: -8 }),
			je("footer", 4200, 4700),
			ze("footer-detail", 4500, 5100)
		]),
		machine: Oc,
		controls: [...yc.map((e) => ({
			id: `focus-${e}`,
			label: bc[e].control,
			event: `FOCUS_${e.toUpperCase()}`,
			group: "Surface",
			description: bc[e].detail,
			activeWhen: {
				var: "surface",
				op: "eq",
				value: e
			}
		})), {
			id: "reset",
			kind: "reset",
			label: "Show all"
		}],
		metadata: { source: "bindings-and-languages/binding-pipeline.svg" }
	})
}, Bc = {
	slug: "bottleneck-lens",
	order: 93,
	title: "State-machine chart lens",
	summary: "Controls dim, highlight, and reinterpret meaningful series in one stable chart.",
	concept: "Stateful quantitative explanation, not a static dashboard screenshot.",
	interaction: "Choose Both, Reads, or Writes to isolate the operational story in the same data.",
	animation: "Both lines draw once; machine states then change emphasis without rebuilding the plot.",
	source: "Kineglyph quantitative example; all values are illustrative.",
	scene: br("bottleneck-lens", {
		title: "The same chart can answer different operational questions",
		description: "A state-machine lens highlights illustrative read or write pressure and updates the interpretation without replacing the chart.",
		metadata: {
			data: "illustrative",
			family: "quantitative"
		}
	}, (e) => {
		let t = e.heading("Select the pressure you want to explain"), n = gc([
			{
				second: 0,
				reads: 18,
				writes: 12
			},
			{
				second: 1,
				reads: 28,
				writes: 19
			},
			{
				second: 2,
				reads: 44,
				writes: 31
			},
			{
				second: 3,
				reads: 58,
				writes: 47
			},
			{
				second: 4,
				reads: 63,
				writes: 68
			},
			{
				second: 5,
				reads: 66,
				writes: 82
			},
			{
				second: 6,
				reads: 71,
				writes: 76
			},
			{
				second: 7,
				reads: 74,
				writes: 69
			}
		], {
			id: "pressure-chart",
			x: "second",
			y: ["reads", "writes"],
			marks: [sc({ curve: "monotone" }), lc({ pointRadius: 3 })],
			title: "Illustrative streaming pressure",
			description: "Read and write pressure over eight seconds. Use the controls to isolate a series and change the interpretation.",
			axes: {
				x: {
					label: "Elapsed time (s)",
					nice: !1
				},
				y: {
					label: "Queue pressure (%)",
					domain: [0, 100],
					nice: !1
				}
			},
			grid: "y",
			legend: { position: "top" },
			seriesBindings: {
				reads: {
					opacity: "readsOpacity",
					highlight: "readsFocus"
				},
				writes: {
					opacity: "writesOpacity",
					highlight: "writesFocus"
				}
			},
			height: {
				wide: 244,
				compact: 214,
				narrow: 180
			},
			motion: "auto",
			duration: 1050
		}), r = e.add(n), i = e.callout("Both queues climb together until write pressure briefly becomes the limiting path.", {
			tone: "info",
			bind: { text: "interpretation" }
		});
		e.stack([
			t,
			r,
			i
		], {
			gap: {
				wide: 18,
				compact: 16,
				narrow: 14
			},
			width: "fill"
		}), e.sequence([
			e.reveal(t, { offset: 8 }),
			e.reveal(r),
			e.reveal(i, { offset: 6 })
		]), e.machine({
			initial: "all",
			states: {
				all: { on: {
					SHOW_READS: "reads",
					SHOW_WRITES: "writes",
					SHOW_ALL: "all"
				} },
				reads: { on: {
					SHOW_READS: "reads",
					SHOW_WRITES: "writes",
					SHOW_ALL: "all"
				} },
				writes: { on: {
					SHOW_READS: "reads",
					SHOW_WRITES: "writes",
					SHOW_ALL: "all"
				} }
			},
			signals: {
				readsOpacity: {
					when: { state: ["all", "reads"] },
					then: 1,
					else: .24
				},
				writesOpacity: {
					when: { state: ["all", "writes"] },
					then: 1,
					else: .24
				},
				readsFocus: {
					when: { state: "reads" },
					then: 1,
					else: 0
				},
				writesFocus: {
					when: { state: "writes" },
					then: 1,
					else: 0
				},
				interpretation: {
					match: { state: !0 },
					cases: {
						reads: "Read pressure rises steadily but stays below 75%; prefetching remains ahead of demand.",
						writes: "Write pressure overtakes reads at 4 s and peaks at 82%; commit throughput is the short-lived bottleneck."
					},
					default: "Both queues climb together until write pressure briefly becomes the limiting path."
				}
			}
		}), e.controls([
			{
				label: "Both",
				event: "SHOW_ALL",
				activeWhen: { state: "all" },
				group: "lens"
			},
			{
				label: "Reads",
				event: "SHOW_READS",
				activeWhen: { state: "reads" },
				group: "lens"
			},
			{
				label: "Writes",
				event: "SHOW_WRITES",
				activeWhen: { state: "writes" },
				group: "lens"
			}
		]);
	})
}, Vc = [
	{
		workload: "Dense volume",
		primitive: "fill_cuboid",
		speedup: 38
	},
	{
		workload: "Dense volume",
		primitive: "set_blocks",
		speedup: 8
	},
	{
		workload: "Dense volume",
		primitive: "prepare + place",
		speedup: 5
	},
	{
		workload: "Dense volume",
		primitive: "BuildingTool.fill",
		speedup: 12
	},
	{
		workload: "Sparse, one id",
		primitive: "fill_cuboid",
		speedup: 1
	},
	{
		workload: "Sparse, one id",
		primitive: "set_blocks",
		speedup: 29
	},
	{
		workload: "Sparse, one id",
		primitive: "prepare + place",
		speedup: 7
	},
	{
		workload: "Sparse, one id",
		primitive: "BuildingTool.fill",
		speedup: 4
	},
	{
		workload: "Mixed ids",
		primitive: "fill_cuboid",
		speedup: 1
	},
	{
		workload: "Mixed ids",
		primitive: "set_blocks",
		speedup: 6
	},
	{
		workload: "Mixed ids",
		primitive: "prepare + place",
		speedup: 21
	},
	{
		workload: "Mixed ids",
		primitive: "BuildingTool.fill",
		speedup: 8
	},
	{
		workload: "Shape + brush",
		primitive: "fill_cuboid",
		speedup: 3
	},
	{
		workload: "Shape + brush",
		primitive: "set_blocks",
		speedup: 4
	},
	{
		workload: "Shape + brush",
		primitive: "prepare + place",
		speedup: 7
	},
	{
		workload: "Shape + brush",
		primitive: "BuildingTool.fill",
		speedup: 24
	}
], Hc = {
	slug: "operation-heatmap",
	order: 92,
	title: "Bulk-operation decision matrix",
	summary: "A responsive heatmap reveals which primitive fits each workload shape.",
	concept: "A genuinely two-dimensional analytical view with exact cell inspection.",
	interaction: "Focus any cell to read its workload, primitive, and illustrative speedup.",
	animation: "Cells sweep across the matrix in reading order, revealing the diagonal pattern.",
	source: "Kineglyph quantitative example; all values are illustrative.",
	scene: br("operation-heatmap", {
		title: "Choose a bulk primitive by workload shape",
		description: "An illustrative heatmap compares the relative speedup of four bulk-write primitives across four workload shapes.",
		metadata: {
			data: "illustrative",
			family: "quantitative"
		}
	}, (e) => {
		let t = e.heading("The diagonal is the design rule"), n = gc(Vc, {
			id: "bulk-decision-matrix",
			marks: dc({
				row: "workload",
				column: "primitive",
				value: "speedup",
				tone: "chart2",
				domain: [0, 40],
				cellLabels: !1,
				format: {
					digits: 0,
					suffix: "×"
				}
			}),
			title: "Illustrative speedup over scalar writes",
			description: "Each cell estimates relative speedup over scalar writes. The best fit lies on the diagonal from dense fills to geometry-aware fills.",
			axes: {
				x: {
					label: "Bulk primitive",
					labelEvery: {
						wide: 1,
						compact: 1,
						narrow: 2
					}
				},
				y: { label: "Workload shape" }
			},
			height: {
				wide: 268,
				compact: 236,
				narrow: 206
			},
			motion: "auto",
			duration: 1050
		}), r = e.add(n), i = e.caption("Illustrative values. Read by row: the darkest cell names the operation shaped for that workload—not a universal winner. Focus a cell for its exact value.");
		e.stack([
			t,
			r,
			i
		], {
			gap: {
				wide: 18,
				compact: 16,
				narrow: 14
			},
			width: "fill"
		}), e.sequence([
			e.reveal(t, { offset: 8 }),
			e.reveal(r),
			e.reveal(i, { offset: 6 })
		]);
	})
};
//#endregion
//#region ../scenes/dist/scenes/nucleation-system.js
function Uc(e) {
	return `FOCUS_${e.replaceAll("-", "_").toUpperCase()}`;
}
function Wc(e, t, n) {
	let r = {
		...Object.fromEntries(t.map((e) => [Uc(e.key), e.key])),
		RESET: "overview"
	}, i = {
		detailTitle: {
			match: { var: "focus" },
			cases: Object.fromEntries(t.map((e) => [e.key, e.title])),
			default: n.title
		},
		detailBody: {
			match: { var: "focus" },
			cases: Object.fromEntries(t.map((e) => [e.key, e.body])),
			default: n.body
		}
	};
	for (let e of t) i[`${e.key}Focus`] = {
		when: {
			var: "focus",
			op: "eq",
			value: e.key
		},
		then: 1,
		else: 0
	}, i[`${e.key}Dim`] = {
		when: {
			var: "focus",
			op: "in",
			value: ["none", e.key]
		},
		then: 1,
		else: .32
	};
	return {
		machine: {
			id: e,
			initial: "overview",
			variables: { focus: "none" },
			states: {
				overview: {
					entry: [{
						type: "set",
						var: "focus",
						value: "none"
					}, {
						type: "select",
						node: null
					}],
					on: r
				},
				...Object.fromEntries(t.map((e) => [e.key, {
					entry: [{
						type: "set",
						var: "focus",
						value: e.key
					}, {
						type: "select",
						node: e.node
					}],
					on: r
				}]))
			},
			signals: i
		},
		controls: [...t.map((t) => ({
			id: `${e}-${t.key}`,
			label: t.label,
			event: Uc(t.key),
			activeWhen: {
				var: "focus",
				op: "eq",
				value: t.key
			}
		})), {
			id: `${e}-reset`,
			kind: "reset",
			label: "Show all"
		}]
	};
}
function Gc(e) {
	return {
		interactive: !0,
		onActivate: Uc(e.key),
		bind: {
			highlight: `${e.key}Focus`,
			opacity: `${e.key}Dim`
		},
		label: e.title,
		description: e.body
	};
}
function Kc(e) {
	return Jn(`${e}-detail`, [Kn(`${e}-detail-mark`, "target", {
		tone: "accent",
		size: 18
	}), z(`${e}-detail-copy`, [Vn(`${e}-detail-title`, "", {
		bind: { text: "detailTitle" },
		width: "fill"
	}), Un(`${e}-detail-body`, "", {
		bind: { text: "detailBody" },
		maxLines: {
			wide: 2,
			compact: 3,
			narrow: 5
		},
		width: "fill"
	})], {
		gap: 2,
		width: "fill"
	})], {
		gap: 12,
		align: "center",
		padding: [11, 14],
		frame: B("inset", { radius: 6 }),
		width: "fill"
	});
}
function qc(e, t, n, r, i = !0) {
	return z(`${e}-root`, [
		Jn(`${e}-head`, [z(`${e}-head-copy`, [Bn(`${e}-label`, t, { tone: "accent" }), Hn(`${e}-title`, n)], {
			gap: 3,
			width: "fill"
		}), Wn(`${e}-stamp`, "NUCLEATION / KINEGLYPH", {
			tone: "muted",
			hidden: {
				wide: !1,
				compact: !0
			}
		})], {
			align: "end",
			justify: "between",
			width: "fill"
		}),
		r,
		...i ? [Kc(e)] : []
	], {
		gap: {
			wide: 20,
			compact: 16,
			narrow: 14
		},
		padding: {
			wide: 24,
			compact: 20,
			narrow: 16
		},
		frame: B("flat"),
		width: "fill"
	});
}
function Jc(e, t, n, r) {
	return z(e, [
		Bn(`${e}-eyebrow`, t),
		Vn(`${e}-name`, n),
		Un(`${e}-note`, r)
	], {
		gap: 2,
		width: "fill"
	});
}
function Yc(e, t, n = "accent") {
	return z(e, [Wn(`${e}-text`, t, {
		tone: n,
		align: "center",
		width: "fill"
	})], {
		padding: [8, 10],
		frame: B("raised", { radius: 4 }),
		align: "center",
		width: "fill"
	});
}
function Xc(e, t = []) {
	let n = e.flatMap((e, t) => Pe(e, 80 + t * 120, 440 + t * 120, {
		offset: 8,
		scale: .985
	})), r = 360 + e.length * 80, i = t.flatMap((e, t) => [...Fe(e, r + t * 140, r + 360 + t * 140), Ie(e, r + 360 + t * 140)]);
	return Ve([...n, ...i], Math.max(1200, r + t.length * 140 + 480));
}
function Zc(e, t, n, r, i, a, o) {
	return {
		slug: t,
		order: e,
		title: n,
		summary: r,
		concept: r,
		interaction: a,
		animation: o,
		source: `${t}.svg`,
		scene: i
	};
}
var Qc = [
	{
		key: "dense",
		label: "Dense",
		node: "fast-dense",
		title: "A dense box is a bounds problem",
		body: "fill_cuboid grows the bounds once and resolves one block id for the entire volume.",
		input: "solid bounds",
		api: "fill_cuboid",
		metric: "1 call",
		tone: "accent"
	},
	{
		key: "sparse",
		label: "Sparse",
		node: "fast-sparse",
		title: "Sparse equal blocks travel as one batch",
		body: "set_blocks crosses the binding once with an array of positions and one parsed descriptor.",
		input: "positions[]",
		api: "set_blocks",
		metric: "1 parse",
		tone: "info"
	},
	{
		key: "mixed",
		label: "Mixed",
		node: "fast-mixed",
		title: "Resolve mixed ids before the hot loop",
		body: "prepare turns block states into palette indices once; place writes those indices in the loop.",
		input: "(pos, id) × N",
		api: "prepare + place",
		metric: "N ids once",
		tone: "warning"
	},
	{
		key: "geometry",
		label: "Geometry",
		node: "fast-geometry",
		title: "Geometry pays for shape and material",
		body: "BuildingTool.fill evaluates the shape and brush per selected cell because both affect the result.",
		input: "shape × brush",
		api: "BuildingTool.fill",
		metric: "per cell",
		tone: "success"
	}
], $c = Wc("fast-workload", Qc, {
	title: "Match the operation to the shape of the input",
	body: "Bulk generation is fastest when the API can resolve bounds, descriptors, or palettes outside the per-cell loop."
});
function el(e, t) {
	let n = Array.from({ length: 7 }, (n, r) => ({
		id: `fast-${e.key}-cell-${r}`,
		type: "rect",
		width: r < 2 + t ? 10 : 6,
		height: 10,
		radius: 2,
		fill: r < 2 + t ? e.tone : "surfaceMuted",
		stroke: "none"
	}));
	return {
		...Jn(`fast-${e.key}`, [
			Jn(`fast-${e.key}-sample`, n, {
				gap: 4,
				width: {
					wide: 112,
					narrow: "fill"
				}
			}),
			Jc(`fast-${e.key}-copy`, "WORKLOAD", e.input, e.metric),
			{
				id: `fast-${e.key}-track`,
				type: "rect",
				width: "fill",
				height: 2,
				fill: e.tone,
				stroke: "none"
			},
			Yc(`fast-${e.key}-api`, e.api, e.tone)
		], {
			gap: {
				wide: 18,
				compact: 12
			},
			align: "center",
			padding: [12, 14],
			frame: B(t === 0 ? "floating" : "raised", { radius: 6 }),
			width: "fill"
		}),
		layout: {
			wide: "row",
			compact: "row",
			narrow: "stack"
		},
		...Gc(e)
	};
}
var tl = Zc(1, "fast-generation", "Fast generation", "Workload fingerprints line up with the bulk call that moves parsing and bounds work out of the loop.", Ct({
	schemaVersion: 2,
	id: "fast-generation",
	title: "Fast schematic generation",
	description: "Four workload shapes aligned with the bulk operation that avoids unnecessary work.",
	breakpoints: {
		wide: 780,
		compact: 520
	},
	background: "canvas",
	root: qc("fast", "FAST GENERATION", "The input already tells you the fast path.", z("fast-lanes", Qc.map(el), {
		gap: 10,
		width: "fill"
	})),
	machine: $c.machine,
	controls: $c.controls,
	timeline: Xc(Qc.map((e) => `fast-${e.key}`)),
	metadata: {
		source: "fast-generation/operation-map.svg",
		revision: 2
	}
}), "Focus a workload to isolate its data shape, cost, and bulk API.", "The four lanes settle into place from the cheapest fixed-cost path to per-cell geometry."), nl = 7, rl = 3;
function il(e, t) {
	let n = [];
	for (let r = 0; r < nl; r += 1) for (let i = 0; i < nl; i += 1) {
		let a = (i - rl) ** 2 + (r - rl) ** 2 <= 9, o = t ? r < 2 ? "warning" : r < 5 ? "accent" : "info" : "info";
		n.push({
			id: `${e}-${r}-${i}`,
			type: "rect",
			width: 15,
			height: 15,
			radius: 2,
			fill: a ? o : "none",
			stroke: a ? o : "border",
			opacity: a ? 1 : .42
		});
	}
	return {
		id: e,
		type: "group",
		layout: "grid",
		columns: nl,
		gap: 3,
		width: 123,
		children: n
	};
}
var al = [
	{
		key: "mask",
		label: "Shape",
		node: "shape-mask-stage",
		title: "Shape answers where",
		body: "The sphere is only a boolean mask over voxel centres. It knows nothing about block states."
	},
	{
		key: "brush",
		label: "Brush",
		node: "shape-brush-stage",
		title: "Brush answers what",
		body: "The brush maps position or field values to block states without deciding which cells exist."
	},
	{
		key: "result",
		label: "Result",
		node: "shape-result-stage",
		title: "fill composes the two contracts",
		body: "BuildingTool.fill visits the selected cells and asks the brush for one block state at each position."
	}
], ol = Wc("shape-composition", al, {
	title: "Geometry and material remain independent",
	body: "A reusable shape can take any brush, and a reusable brush can paint any bounded shape."
});
function sl(e, t, n, r) {
	return {
		...z(e.node, [t, Jc(`${e.node}-copy`, n, e.label, r)], {
			gap: 12,
			align: "center",
			padding: [18, 16],
			frame: B(e.key === "result" ? "floating" : "raised", { radius: 8 }),
			width: "fill"
		}),
		...Gc(e)
	};
}
var cl = z("shape-brush-ramp", [
	"warning",
	"warning",
	"accent",
	"accent",
	"success",
	"info",
	"info"
].map((e, t) => ({
	id: `shape-brush-swatch-${t}`,
	type: "rect",
	width: 123,
	height: 15,
	radius: 2,
	fill: e,
	stroke: "none"
})), {
	gap: 3,
	width: 123
}), ll = Zc(2, "shapes-and-brushes", "Shapes and brushes", "A mask and a material rule remain independent until fill composes them into block states.", Ct({
	schemaVersion: 2,
	id: "shapes-and-brushes",
	title: "Shapes and brushes",
	description: "A voxel mask and a material ramp compose into a filled, coloured schematic slice.",
	breakpoints: {
		wide: 760,
		compact: 520
	},
	background: "canvas",
	root: qc("shape", "SHAPES + BRUSHES", "Where and what are separate decisions.", {
		id: "shape-compositor",
		type: "group",
		layout: {
			wide: "row",
			compact: "stack"
		},
		gap: {
			wide: 18,
			compact: 12
		},
		align: "stretch",
		width: "fill",
		children: [
			sl(al[0], il("shape-mask-grid", !1), "MASK", "sphere(c, 3)"),
			z("shape-plus", [Hn("shape-plus-symbol", "+", {
				tone: "accent",
				align: "center",
				width: "fill"
			})], {
				justify: "center",
				width: {
					wide: 38,
					compact: "fill"
				}
			}),
			sl(al[1], cl, "MATERIAL RULE", "field → palette"),
			z("shape-equals", [Hn("shape-equals-symbol", "=", {
				tone: "accent",
				align: "center",
				width: "fill"
			})], {
				justify: "center",
				width: {
					wide: 38,
					compact: "fill"
				}
			}),
			sl(al[2], il("shape-result-grid", !0), "SCHEMATIC", "24 selected cells")
		]
	}),
	machine: ol.machine,
	controls: ol.controls,
	timeline: Xc([
		"shape-mask-stage",
		"shape-plus",
		"shape-brush-stage",
		"shape-equals",
		"shape-result-stage"
	]),
	metadata: {
		source: "shapes-brushes/shape-brush-map.svg",
		revision: 2
	}
}), "Inspect the mask, material ramp, or resulting voxel slice.", "The composition reads left to right and the finished slice arrives last."), ul = [
	{
		key: "field",
		label: "Field",
		node: "sdf-field",
		title: "One immutable scalar field",
		body: "Field3 returns one number for each position. Geometry and material read that same value."
	},
	{
		key: "geometry",
		label: "Geometry",
		node: "sdf-geometry",
		title: "The zero crossing becomes occupancy",
		body: "SDF composition and displacement turn the sampled scalar into a bounded solid."
	},
	{
		key: "material",
		label: "Material",
		node: "sdf-material",
		title: "The field also drives block choice",
		body: "A field brush maps the same scalar to a palette, keeping colour aligned with the surface."
	},
	{
		key: "schematic",
		label: "Schematic",
		node: "sdf-schematic",
		title: "Both branches meet at fill",
		body: "The result is ordinary editable schematic data, not a separate render-only approximation."
	}
], dl = Wc("sdf-flow", ul, {
	title: "A scalar can shape geometry and material at once",
	body: "Keeping both branches on the same field avoids the drift caused by unrelated noise functions."
});
function fl(e) {
	return {
		SHAPE_BLOOM: {
			target: e,
			actions: [{
				type: "set",
				var: "shape",
				value: "bloom"
			}]
		},
		SHAPE_RINGS: {
			target: e,
			actions: [{
				type: "set",
				var: "shape",
				value: "rings"
			}]
		},
		SHAPE_FRAME: {
			target: e,
			actions: [{
				type: "set",
				var: "shape",
				value: "frame"
			}]
		},
		MATERIAL_FIELD: {
			target: e,
			actions: [{
				type: "set",
				var: "material",
				value: "field"
			}]
		},
		MATERIAL_CALCITE: {
			target: e,
			actions: [{
				type: "set",
				var: "material",
				value: "calcite"
			}]
		},
		MATERIAL_COPPER: {
			target: e,
			actions: [{
				type: "set",
				var: "material",
				value: "copper"
			}]
		}
	};
}
var pl = {
	...dl.machine,
	variables: {
		...dl.machine.variables,
		shape: "bloom",
		material: "field"
	},
	states: Object.fromEntries(Object.entries(dl.machine.states).map(([e, t]) => [e, {
		...t,
		on: {
			...t.on,
			...fl(e)
		}
	}]))
}, ml = [
	{
		id: "sdf-shape-bloom",
		label: "Bloom",
		event: "SHAPE_BLOOM",
		group: "Shape",
		activeWhen: {
			var: "shape",
			op: "eq",
			value: "bloom"
		}
	},
	{
		id: "sdf-shape-rings",
		label: "Rings",
		event: "SHAPE_RINGS",
		group: "Shape",
		activeWhen: {
			var: "shape",
			op: "eq",
			value: "rings"
		}
	},
	{
		id: "sdf-shape-frame",
		label: "Frame",
		event: "SHAPE_FRAME",
		group: "Shape",
		activeWhen: {
			var: "shape",
			op: "eq",
			value: "frame"
		}
	},
	{
		id: "sdf-material-field",
		label: "Field ramp",
		event: "MATERIAL_FIELD",
		group: "Material",
		activeWhen: {
			var: "material",
			op: "eq",
			value: "field"
		}
	},
	{
		id: "sdf-material-calcite",
		label: "Calcite",
		event: "MATERIAL_CALCITE",
		group: "Material",
		activeWhen: {
			var: "material",
			op: "eq",
			value: "calcite"
		}
	},
	{
		id: "sdf-material-copper",
		label: "Copper",
		event: "MATERIAL_COPPER",
		group: "Material",
		activeWhen: {
			var: "material",
			op: "eq",
			value: "copper"
		}
	}
];
function hl() {
	return {
		...z("sdf-field", [{
			id: "sdf-contours",
			type: "group",
			layout: "overlay",
			width: 170,
			height: 150,
			align: "center",
			justify: "center",
			children: [
				68,
				52,
				36,
				20
			].map((e, t) => ({
				id: `sdf-contour-${t}`,
				type: "circle",
				radius: e,
				fill: t === 3 ? mt([{
					at: 0,
					color: "accent",
					opacity: .8
				}, {
					at: 1,
					color: "accent",
					opacity: .08
				}]) : "none",
				stroke: t % 2 == 0 ? "accent" : "info",
				strokeWidth: 1.5,
				dash: t === 1 ? "dashed" : "solid"
			}))
		}, Jc("sdf-field-copy", "FIELD3", "f(x, y, z)", "immutable scalar")], {
			gap: 12,
			align: "center",
			padding: 18,
			frame: B("glass", { radius: 10 }),
			width: "fill"
		}),
		...Gc(ul[0])
	};
}
function gl(e, t) {
	let n = t === "geometry" ? {
		id: "sdf-geometry-shape",
		type: "path",
		d: "M 12 66 C 22 16 52 8 76 30 C 96 48 112 24 128 46 C 146 70 120 110 82 116 C 40 124 4 102 12 66 Z",
		viewBox: {
			width: 150,
			height: 130
		},
		width: 150,
		height: 120,
		fill: ht("info", {
			from: .5,
			to: .08,
			angle: 90
		}),
		stroke: "info",
		strokeWidth: 2
	} : z("sdf-material-ramp", [
		"warning",
		"success",
		"accent",
		"info"
	].map((e, t) => ({
		id: `sdf-material-${t}`,
		type: "rect",
		width: 150,
		height: 24,
		fill: e,
		stroke: "none"
	})), {
		gap: 4,
		width: 150
	});
	return {
		...z(e.node, [n, Jc(`${e.node}-copy`, t.toUpperCase(), e.label, t === "geometry" ? "d ≤ 0" : "field → block")], {
			gap: 10,
			align: "center",
			padding: 16,
			frame: B("raised", { radius: 8 }),
			width: "fill"
		}),
		...Gc(e)
	};
}
var _l = {
	...z("sdf-schematic", [{
		id: "sdf-live-build",
		type: "image",
		src: "data:image/svg+xml,%3Csvg%20xmlns%3D%22http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg%22%20viewBox%3D%220%200%20240%20176%22%3E%3Cg%20fill%3D%22none%22%20stroke%3D%22%238994a3%22%20stroke-width%3D%222%22%3E%3Cpath%20d%3D%22M120%2019%20205%2062v67l-85%2042-85-42V62Z%22%20opacity%3D%22.34%22%2F%3E%3Cpath%20d%3D%22m35%2062%2085%2043%2085-43M120%20105v66%22%20opacity%3D%22.28%22%2F%3E%3C%2Fg%3E%3Cg%20fill%3D%22%238994a3%22%3E%3Crect%20x%3D%2274%22%20y%3D%2265%22%20width%3D%2224%22%20height%3D%2224%22%20rx%3D%223%22%20opacity%3D%22.45%22%2F%3E%3Crect%20x%3D%22100%22%20y%3D%2251%22%20width%3D%2224%22%20height%3D%2224%22%20rx%3D%223%22%20opacity%3D%22.7%22%2F%3E%3Crect%20x%3D%22126%22%20y%3D%2266%22%20width%3D%2224%22%20height%3D%2224%22%20rx%3D%223%22%20opacity%3D%22.52%22%2F%3E%3Crect%20x%3D%22100%22%20y%3D%2279%22%20width%3D%2224%22%20height%3D%2224%22%20rx%3D%223%22%20opacity%3D%22.86%22%2F%3E%3Crect%20x%3D%22126%22%20y%3D%2294%22%20width%3D%2224%22%20height%3D%2224%22%20rx%3D%223%22%20opacity%3D%22.7%22%2F%3E%3Crect%20x%3D%2274%22%20y%3D%2294%22%20width%3D%2224%22%20height%3D%2224%22%20rx%3D%223%22%20opacity%3D%22.6%22%2F%3E%3C%2Fg%3E%3Ccircle%20cx%3D%22120%22%20cy%3D%22105%22%20r%3D%2254%22%20fill%3D%22none%22%20stroke%3D%22%2362d4c3%22%20stroke-width%3D%222%22%20stroke-dasharray%3D%224%207%22%20opacity%3D%22.8%22%2F%3E%3C%2Fsvg%3E",
		alt: "Interactive Minecraft schematic generated from the selected SDF",
		fit: "contain",
		live: !0,
		width: {
			wide: 220,
			compact: 180,
			narrow: "fill"
		},
		height: {
			wide: 176,
			compact: 150,
			narrow: 210
		},
		radius: 8
	}, Jc("sdf-result-copy", "LIVE · WASM", "Schematic", "drag to inspect")], {
		gap: 10,
		align: "center",
		padding: {
			wide: 14,
			compact: 12,
			narrow: 14
		},
		frame: B("raised", { radius: 10 }),
		width: "fill"
	}),
	...Gc(ul[3])
}, vl = [
	{
		id: "sdf-field-geometry",
		from: {
			node: "sdf-field",
			side: {
				wide: "right",
				compact: "bottom"
			},
			offset: .35
		},
		to: {
			node: "sdf-geometry",
			side: {
				wide: "left",
				compact: "top"
			}
		},
		route: "arc",
		head: "diamond",
		stroke: "flow",
		packets: {
			count: 1,
			period: 1800,
			tone: "info"
		}
	},
	{
		id: "sdf-field-material",
		from: {
			node: "sdf-field",
			side: {
				wide: "right",
				compact: "bottom"
			},
			offset: .7
		},
		to: {
			node: "sdf-material",
			side: {
				wide: "left",
				compact: "top"
			}
		},
		route: "orthogonal",
		head: "dot",
		stroke: "flow",
		packets: {
			count: 1,
			period: 1800,
			tone: "accent"
		}
	},
	{
		id: "sdf-geometry-result",
		from: {
			node: "sdf-geometry",
			side: {
				wide: "right",
				compact: "bottom"
			}
		},
		to: {
			node: "sdf-schematic",
			side: {
				wide: "left",
				compact: "top"
			},
			offset: .35
		},
		route: "straight",
		head: "triangle"
	},
	{
		id: "sdf-material-result",
		from: {
			node: "sdf-material",
			side: {
				wide: "right",
				compact: "bottom"
			}
		},
		to: {
			node: "sdf-schematic",
			side: {
				wide: "left",
				compact: "top"
			},
			offset: .7
		},
		route: "curve",
		head: "triangle"
	}
], yl = Zc(3, "sdf-and-fields", "SDF and fields", "One scalar field bifurcates into occupancy and material, then recombines as editable blocks.", Ct({
	schemaVersion: 2,
	id: "sdf-and-fields",
	title: "SDFs and scalar fields",
	description: "One scalar field splits into a geometry branch and a material branch before fill recombines them.",
	breakpoints: {
		wide: 900,
		compact: 520
	},
	background: "canvas",
	root: qc("sdf", "SDF + FIELDS", "One number. Two readings. One build.", {
		id: "sdf-map",
		type: "group",
		layout: {
			wide: "row",
			compact: "row",
			narrow: "stack"
		},
		gap: {
			wide: 64,
			compact: 30,
			narrow: 34
		},
		align: "stretch",
		width: "fill",
		children: [
			hl(),
			z("sdf-branches", [gl(ul[1], "geometry"), gl(ul[2], "material")], {
				gap: 12,
				width: "fill"
			}),
			_l
		]
	}),
	edges: vl,
	machine: pl,
	controls: ml,
	timeline: Xc([
		"sdf-field",
		"sdf-geometry",
		"sdf-material",
		"sdf-schematic"
	], vl.map((e) => e.id)),
	metadata: {
		source: "sdf-and-fields/sdf-field-pipeline.svg",
		revision: 2
	}
}), "Choose a volume and material, then drag the generated schematic to inspect the result.", "Contours arrive first, the two interpretations split apart, and fill closes the loop."), bl = [
	{
		key: "target",
		label: "Target",
		node: "color-target",
		title: "Start with a measured target",
		body: "The input is a concrete colour in sRGB; comparison happens after conversion to Oklab."
	},
	{
		key: "lab",
		label: "Oklab",
		node: "color-lab",
		title: "Distance is perceptual",
		body: "Oklab makes similar numerical distances read more like similar visible differences."
	},
	{
		key: "palette",
		label: "Palette",
		node: "color-palette",
		title: "The palette is a constraint",
		body: "Filters remove unavailable, unsafe, or unwanted blocks before nearest-colour search."
	},
	{
		key: "methods",
		label: "Methods",
		node: "color-methods",
		title: "Selection method changes the texture",
		body: "Nearest, ramps, gradients, and dithering trade exact local colour for continuity or pattern."
	}
], xl = Wc("color-laboratory", bl, {
	title: "Colour selection is measurement under constraints",
	body: "Convert the target, filter the available blocks, then choose a selection strategy for the surface."
}), Sl = Zc(4, "palettes-and-color", "Palettes and colour", "A colour laboratory separates perceptual measurement, palette constraints, and surface strategy.", Ct({
	schemaVersion: 2,
	id: "palettes-and-color",
	title: "Palettes and colour",
	description: "A target colour passes through perceptual measurement and a constrained block palette before four selection methods.",
	breakpoints: {
		wide: 820,
		compact: 540
	},
	background: "canvas",
	root: qc("color", "PALETTES + COLOUR", "Measure first. Constrain second. Choose texture last.", {
		id: "color-lab-bench",
		type: "group",
		layout: {
			wide: "row",
			compact: "grid",
			narrow: "stack"
		},
		columns: {
			compact: 2,
			narrow: 1
		},
		gap: 12,
		align: "stretch",
		width: "fill",
		children: [
			{
				...z("color-target", [{
					id: "color-target-swatch",
					type: "rect",
					width: "fill",
					height: 132,
					radius: 8,
					fill: pt([
						{
							at: 0,
							color: "warning"
						},
						{
							at: .52,
							color: "danger"
						},
						{
							at: 1,
							color: "accent"
						}
					], { angle: 135 }),
					stroke: "none"
				}, Jc("color-target-copy", "INPUT", "#D78368", "sRGB target")], {
					gap: 12,
					padding: 14,
					frame: B("floating", { radius: 8 }),
					width: "fill"
				}),
				...Gc(bl[0])
			},
			{
				...z("color-lab", [{
					id: "color-lab-disc",
					type: "group",
					layout: "overlay",
					width: 138,
					height: 138,
					align: "center",
					justify: "center",
					children: [
						{
							id: "color-lab-outer",
							type: "circle",
							radius: 64,
							fill: mt([
								{
									at: 0,
									color: "surface",
									opacity: .1
								},
								{
									at: .7,
									color: "info",
									opacity: .16
								},
								{
									at: 1,
									color: "accent",
									opacity: .55
								}
							]),
							stroke: "accent"
						},
						{
							id: "color-lab-x",
							type: "rect",
							width: 112,
							height: 1,
							fill: "border",
							stroke: "none"
						},
						{
							id: "color-lab-y",
							type: "rect",
							width: 1,
							height: 112,
							fill: "border",
							stroke: "none"
						},
						{
							id: "color-lab-point",
							type: "circle",
							radius: 7,
							fill: "warning",
							stroke: "surface",
							strokeWidth: 2
						}
					]
				}, Jc("color-lab-copy", "SPACE", "Oklab", "ΔE comparison")], {
					gap: 10,
					align: "center",
					padding: 14,
					frame: B("raised", { radius: 8 }),
					width: "fill"
				}),
				...Gc(bl[1])
			},
			{
				...z("color-palette", [Jn("color-blocks", [
					"warning",
					"danger",
					"accent",
					"success",
					"info",
					"chart2"
				].map((e, t) => ({
					id: `color-block-${t}`,
					type: "rect",
					width: "fill",
					height: 64 + t % 3 * 12,
					radius: 3,
					fill: e,
					stroke: "none",
					opacity: t === 1 ? .35 : 1
				})), {
					gap: 5,
					align: "end",
					width: "fill"
				}), Jc("color-palette-copy", "FILTER", "Block palette", "available candidates")], {
					gap: 12,
					padding: 14,
					frame: B("inset", { radius: 8 }),
					width: "fill"
				}),
				...Gc(bl[2])
			},
			{
				...z("color-methods", [
					["nearest", "one block"],
					["ramp", "ordered set"],
					["gradient", "continuous"],
					["dither", "spatial mix"]
				].map(([e, t], n) => Jn(`color-method-${e}`, [
					{
						id: `color-method-${e}-mark`,
						type: "circle",
						radius: 5 + n,
						fill: bl[n % bl.length]?.key === "lab" ? "info" : "accent",
						stroke: "none"
					},
					Vn(`color-method-${e}-name`, e ?? ""),
					Un(`color-method-${e}-note`, t ?? "", {
						align: "end",
						width: "fill"
					})
				], {
					gap: 9,
					align: "center",
					width: "fill"
				})), {
					gap: 10,
					padding: 16,
					frame: B("raised", { radius: 8 }),
					width: "fill"
				}),
				...Gc(bl[3])
			}
		]
	}),
	machine: xl.machine,
	controls: xl.controls,
	timeline: Xc([
		"color-target",
		"color-lab",
		"color-palette",
		"color-methods"
	]),
	metadata: {
		source: "palettes-and-color/color-pipeline.svg",
		revision: 2
	}
}), "Inspect each bench instrument to see the contract it owns.", "The target, Oklab disc, block palette, and methods appear in the order data reaches them."), Cl = [
	{
		key: "signal",
		label: "Signal",
		node: "sim-signal",
		title: "Known comparator strength needs no world",
		body: "signal(0..15) writes the shorthand state directly when only the encoded level matters.",
		question: "Need a level?",
		answer: "signal",
		tone: "accent"
	},
	{
		key: "placement",
		label: "Placement",
		node: "sim-placement",
		title: "Neighbour-derived state needs placement context",
		body: "Simulated placement resolves stairs, fences, rails, and other states that depend on nearby blocks.",
		question: "Need derived state?",
		answer: "simulate placement",
		tone: "info"
	},
	{
		key: "circuit",
		label: "Circuit",
		node: "sim-circuit",
		title: "Circuit truth belongs to MCHPRS",
		body: "Use the circuit engine when the question is redstone output rather than general world evolution.",
		question: "Need circuit output?",
		answer: "MCHPRS",
		tone: "warning"
	},
	{
		key: "world",
		label: "World",
		node: "sim-world",
		title: "World evolution needs ticks",
		body: "TickSimulation handles scheduled updates, fluids, entities, pistons, and temporal causality.",
		question: "Need time?",
		answer: "TickSimulation",
		tone: "success"
	}
], wl = Wc("simulation-choice", Cl, {
	title: "Choose the smallest engine that answers the question",
	body: "Direct shorthand, placement context, circuit execution, and world ticks solve different classes of state."
});
function Tl(e, t) {
	return {
		...Jn(e.node, [
			z(`${e.node}-index`, [Wn(`${e.node}-index-text`, String(t + 1).padStart(2, "0"), {
				tone: e.tone,
				align: "center",
				width: "fill"
			})], {
				width: 42,
				padding: 9,
				frame: B("inset", { radius: 21 }),
				align: "center"
			}),
			Jc(`${e.node}-question`, "QUESTION", e.question, e.key === "world" ? "temporal" : "state"),
			{
				id: `${e.node}-line`,
				type: "rect",
				width: "fill",
				height: 1,
				fill: e.tone,
				stroke: "none"
			},
			Yc(`${e.node}-answer`, e.answer, e.tone)
		], {
			gap: 14,
			align: "center",
			padding: [10, 12],
			width: "fill"
		}),
		layout: {
			wide: "row",
			compact: "row",
			narrow: "stack"
		},
		...Gc(e)
	};
}
var El = Zc(5, "smart-simulation", "Placement and simulation", "A four-question instrument selects the smallest state model that can answer the job.", Ct({
	schemaVersion: 2,
	id: "smart-simulation",
	title: "Choosing a simulation surface",
	description: "Four questions lead from direct signal state through placement and circuit execution to full tick simulation.",
	breakpoints: {
		wide: 780,
		compact: 520
	},
	background: "canvas",
	root: qc("sim", "PLACEMENT + SIMULATION", "Ask what must be true, then pay only for that model.", z("sim-instrument", Cl.map(Tl), {
		gap: 4,
		padding: [10, 12],
		frame: B("raised", { radius: 10 }),
		width: "fill"
	})),
	machine: wl.machine,
	controls: wl.controls,
	timeline: Xc(Cl.map((e) => e.node)),
	metadata: {
		source: "smart-simulation/choose-engine.svg",
		revision: 2
	}
}), "Focus a question to see why its engine is sufficient.", "The instrument advances from direct state to full temporal simulation."), Dl = [
	{
		key: "detect",
		label: "Detect",
		node: "format-inputs",
		title: "Content detection precedes parsing",
		body: "Bytes and container structure select the parser; a filename extension is only a hint."
	},
	{
		key: "model",
		label: "Model",
		node: "format-model",
		title: "Every parser converges on one editable model",
		body: "Blocks, entities, metadata, regions, and bounds use the same in-memory schematic contract."
	},
	{
		key: "export",
		label: "Export",
		node: "format-outputs",
		title: "Export is an explicit destination choice",
		body: "Structure, snapshot, and world formats keep their own capabilities and loss boundaries visible."
	}
], Ol = Wc("format-hub", Dl, {
	title: "Many containers, one model, explicit destinations",
	body: "Nucleation isolates format quirks at the edge so edits and analysis operate on one schematic representation."
});
function kl(e, t, n) {
	return z(e, [Wn(`${e}-text`, t, {
		tone: n,
		align: "center",
		width: "fill"
	})], {
		padding: [10, 8],
		frame: B("raised", { radius: 4 }),
		width: "fill"
	});
}
var Al = {
	...z("format-inputs", [Bn("format-inputs-label", "DETECT + PARSE"), {
		id: "format-input-grid",
		type: "group",
		layout: "grid",
		columns: {
			wide: 3,
			narrow: 2
		},
		gap: 7,
		width: "fill",
		children: [
			".schem",
			".litematic",
			".mcstructure",
			".nusn",
			".snbt",
			"world/"
		].map((e, t) => kl(`format-in-${t}`, e, t % 2 == 0 ? "info" : "accent"))
	}], {
		gap: 10,
		padding: 16,
		frame: B("raised", { radius: 8 }),
		width: "fill"
	}),
	...Gc(Dl[0])
}, jl = {
	...z("format-model", [
		Kn("format-model-cube", "cube", {
			tone: "accent",
			size: 84
		}),
		Hn("format-model-title", "Schematic", {
			align: "center",
			width: "fill"
		}),
		Un("format-model-note", "blocks · entities · metadata · regions", {
			align: "center",
			width: "fill",
			maxLines: 2
		})
	], {
		gap: 8,
		align: "center",
		padding: [24, 18],
		frame: B("glass", { radius: 12 }),
		width: "fill"
	}),
	...Gc(Dl[1])
}, Ml = {
	...z("format-outputs", [Bn("format-outputs-label", "EXPORT"), ...[
		["STRUCTURE", ".schem · .litematic"],
		["SNAPSHOT", ".nusn · .snbt"],
		["WORLD", "region · chunk"]
	].map(([e, t], n) => Jn(`format-out-${n}`, [Vn(`format-out-${n}-name`, e ?? ""), Wn(`format-out-${n}-note`, t ?? "", {
		tone: n === 2 ? "success" : "accent",
		align: "end",
		width: "fill"
	})], {
		gap: 12,
		align: "center",
		width: "fill"
	}))], {
		gap: 12,
		padding: 16,
		frame: B("raised", { radius: 8 }),
		width: "fill"
	}),
	...Gc(Dl[2])
}, Nl = [{
	id: "format-read",
	from: {
		node: "format-inputs",
		side: {
			wide: "right",
			compact: "bottom"
		}
	},
	to: {
		node: "format-model",
		side: {
			wide: "left",
			compact: "top"
		}
	},
	route: "straight",
	head: "arrow",
	tail: "dot",
	stroke: "flow",
	label: "detect",
	packets: {
		count: 2,
		period: 1700
	}
}, {
	id: "format-write",
	from: {
		node: "format-model",
		side: {
			wide: "right",
			compact: "bottom"
		}
	},
	to: {
		node: "format-outputs",
		side: {
			wide: "left",
			compact: "top"
		}
	},
	route: "orthogonal",
	head: "bar",
	stroke: "dashed",
	packets: {
		count: 2,
		period: 1700
	}
}], Pl = Zc(6, "formats-and-io", "Formats and I/O", "A compact format hub separates container detection, the editable model, and explicit export.", Ct({
	schemaVersion: 2,
	id: "formats-and-io",
	title: "Formats and I/O",
	description: "Container detection and parsers converge on one editable schematic model before explicit export branches out again.",
	breakpoints: {
		wide: 900,
		compact: 520
	},
	background: "canvas",
	root: qc("format", "FORMATS + I/O", "Format quirks stay at the boundary.", {
		id: "format-map",
		type: "group",
		layout: {
			wide: "row",
			compact: "stack"
		},
		gap: {
			wide: 36,
			compact: 34
		},
		align: "stretch",
		width: "fill",
		children: [
			Al,
			jl,
			Ml
		]
	}),
	edges: Nl,
	machine: Ol.machine,
	controls: Ol.controls,
	timeline: Xc([
		"format-inputs",
		"format-model",
		"format-outputs"
	], Nl.map((e) => e.id)),
	metadata: {
		source: "formats-and-io/format-pipeline.svg",
		revision: 2
	}
}), "Focus ingress, the model, or egress to inspect the boundary.", "Formats converge on the model, then flow back out through deliberate export paths."), Fl = [
	{
		key: "javascript",
		label: "JS / TS",
		node: "binding-javascript",
		title: "JavaScript / TypeScript · WASM",
		body: "The browser and Node surfaces share generated names plus byte and JSON contracts.",
		runtime: "WASM",
		tone: "accent"
	},
	{
		key: "python",
		label: "Python",
		node: "binding-python",
		title: "Python · nanobind",
		body: "Python objects wrap the native core through a generated, typed extension surface.",
		runtime: "nanobind",
		tone: "info"
	},
	{
		key: "kotlin",
		label: "Kotlin",
		node: "binding-kotlin",
		title: "Kotlin · JNA",
		body: "JVM callers load the shared library and use generated symbol and payload definitions.",
		runtime: "JNA",
		tone: "warning"
	},
	{
		key: "php",
		label: "PHP",
		node: "binding-php",
		title: "PHP · FFI",
		body: "PHP binds the stable C ABI at runtime without a hand-maintained parallel API.",
		runtime: "FFI",
		tone: "success"
	},
	{
		key: "c",
		label: "C",
		node: "binding-c",
		title: "C · stable headers",
		body: "The C ABI is the lowest common contract used by foreign-language packages.",
		runtime: "ABI",
		tone: "danger"
	},
	{
		key: "cpp",
		label: "C++",
		node: "binding-cpp",
		title: "C++ · typed wrappers",
		body: "RAII and native types wrap the generated C symbols without duplicating the core.",
		runtime: "RAII",
		tone: "accent"
	},
	{
		key: "rust",
		label: "Rust",
		node: "binding-rust",
		title: "Rust · direct crate",
		body: "Rust callers bypass the bridge and call the implementation directly.",
		runtime: "native",
		tone: "warning"
	}
], Il = Wc("binding-surfaces-v2", Fl, {
	title: "One implementation, generated foreign surfaces",
	body: "The annotated bridge owns naming and transport contracts; native Rust remains a direct call path."
});
function Ll(e) {
	return {
		...z(e.node, [Vn(`${e.node}-name`, e.label), Wn(`${e.node}-runtime`, e.runtime, { tone: e.tone })], {
			gap: 3,
			padding: [10, 12],
			frame: B("raised", { radius: 4 }),
			width: "fill"
		}),
		...Gc(e)
	};
}
Zc(7, "bindings-and-languages", "Bindings and languages", "A physical stack distinguishes the native core, the generated bridge contract, and each package surface.", Ct({
	schemaVersion: 2,
	id: "bindings-and-languages",
	title: "Bindings and languages",
	description: "The Rust implementation feeds an annotated bridge and six generated foreign-language packages while native Rust remains direct.",
	breakpoints: {
		wide: 780,
		compact: 520
	},
	background: "canvas",
	root: qc("binding", "BINDINGS", "One core. One bridge contract. Seven surfaces.", z("binding-foundation", [
		{ ...Jn("binding-core-row", [z("binding-core", [Bn("binding-core-label", "IMPLEMENTATION"), Hn("binding-core-title", "Rust core")], {
			gap: 3,
			padding: [18, 20],
			frame: B("floating", { radius: 8 }),
			width: "fill"
		}), Ll(Fl[6])], {
			gap: 12,
			align: "stretch",
			width: "fill"
		}) },
		z("binding-bridge", [Jn("binding-bridge-title", [Kn("binding-bridge-icon", "layers", {
			tone: "accent",
			size: 22
		}), Vn("binding-bridge-name", "src/bridge · annotated contract")], {
			gap: 10,
			align: "center",
			width: "fill"
		}), Un("binding-bridge-note", "names · bytes · JSON · errors", { width: "fill" })], {
			gap: 4,
			padding: [14, 18],
			frame: B("glass", { radius: 6 }),
			width: "fill"
		}),
		{
			id: "binding-tabs",
			type: "group",
			layout: "grid",
			columns: {
				wide: 6,
				compact: 3,
				narrow: 2
			},
			gap: 8,
			width: "fill",
			children: Fl.slice(0, 6).map(Ll)
		}
	], {
		gap: 12,
		width: "fill"
	})),
	machine: Il.machine,
	controls: Il.controls,
	timeline: Xc([
		"binding-core-row",
		"binding-bridge",
		"binding-tabs"
	]),
	metadata: {
		source: "bindings-and-languages/binding-pipeline.svg",
		revision: 2
	}
}), "Choose a language tab to inspect its transport without losing the shared architecture.", "The core lands first, the bridge settles above it, and the generated surfaces fan out last.");
var Rl = [
	{
		key: "opaque",
		label: "Opaque",
		node: "mesh-opaque",
		title: "Opaque geometry writes depth first",
		body: "Solid faces draw first and establish the depth buffer for cheaper rejection behind them."
	},
	{
		key: "cutout",
		label: "Cutout",
		node: "mesh-cutout",
		title: "Cutout geometry discards empty texels",
		body: "Leaves and panes keep hard alpha edges without blending while still using depth testing."
	},
	{
		key: "transparent",
		label: "Transparent",
		node: "mesh-transparent",
		title: "Transparent geometry blends last",
		body: "Water and stained glass are sorted back to front after the depth-writing layers."
	},
	{
		key: "portable",
		label: "3D data",
		node: "mesh-portable",
		title: "Portable output keeps the mesh as data",
		body: "GLB, glTF, USDZ, and NUCM feed viewers, DCC tools, and caches."
	},
	{
		key: "pixels",
		label: "Pixels",
		node: "mesh-pixels",
		title: "The native renderer turns the same mesh into pixels",
		body: "Camera, grid, materials, and lighting produce stills or deterministic animation frames."
	}
], zl = Wc("mesh-layers-v2", Rl, {
	title: "Mesh once, then keep the geometry or draw it",
	body: "The mesher builds three ordered layers over one atlas; export and native rendering share that result."
});
function Bl(e, t, n, r) {
	return {
		...z(e.node, [Jn(`${e.node}-copy`, [Wn(`${e.node}-order`, r, { tone: t }), Vn(`${e.node}-name`, e.label)], {
			gap: 12,
			align: "center",
			width: "fill"
		}), {
			id: `${e.node}-slab`,
			type: "rect",
			width: n,
			height: 18,
			radius: 3,
			fill: t,
			stroke: t
		}], {
			gap: 8,
			align: "start",
			padding: [10, 12],
			frame: B("raised", { radius: 5 }),
			width: "fill"
		}),
		...Gc(e)
	};
}
function Vl(e, t, n, r) {
	return {
		...z(e.node, [
			Kn(`${e.node}-icon`, t, {
				tone: r,
				size: 36
			}),
			Vn(`${e.node}-name`, e.label),
			Wn(`${e.node}-formats`, n, { tone: r })
		], {
			gap: 7,
			align: "center",
			padding: [18, 16],
			frame: B("floating", { radius: 8 }),
			width: "fill"
		}),
		...Gc(e)
	};
}
var Hl = [{
	id: "mesh-to-portable",
	from: {
		node: "mesh-stack",
		side: {
			wide: "right",
			compact: "bottom"
		},
		offset: .35
	},
	to: {
		node: "mesh-portable",
		side: {
			wide: "left",
			compact: "top"
		}
	},
	route: "arc",
	head: "arrow",
	stroke: "flow",
	packets: {
		count: 1,
		period: 1800
	}
}, {
	id: "mesh-to-pixels",
	from: {
		node: "mesh-stack",
		side: {
			wide: "right",
			compact: "bottom"
		},
		offset: .7
	},
	to: {
		node: "mesh-pixels",
		side: {
			wide: "left",
			compact: "top"
		}
	},
	route: "curve",
	head: "arrow",
	stroke: "flow",
	packets: {
		count: 1,
		period: 1800
	}
}], Ul = Zc(8, "meshing-and-rendering", "Meshing and rendering", "Exploded mesh slabs make draw order tangible before the shared geometry branches to data or pixels.", Ct({
	schemaVersion: 2,
	id: "meshing-and-rendering",
	title: "Meshing and rendering",
	description: "Three explicit mesh layers over one texture atlas branch to portable geometry data or native rendered pixels.",
	breakpoints: {
		wide: 900,
		compact: 520
	},
	background: "canvas",
	root: qc("mesh", "MESHING + RENDERING", "Transparency order is part of the data path.", {
		id: "mesh-map",
		type: "group",
		layout: {
			wide: "row",
			compact: "stack"
		},
		gap: {
			wide: 44,
			compact: 34
		},
		align: "stretch",
		width: "fill",
		children: [z("mesh-stack", [
			Jn("mesh-input", [Kn("mesh-input-cube", "cube", {
				tone: "info",
				size: 24
			}), Vn("mesh-input-title", "Schematic + resource pack")], {
				gap: 10,
				align: "center",
				width: "fill"
			}),
			Bl(Rl[0], "accent", "100%", "01"),
			Bl(Rl[1], "warning", "76%", "02"),
			Bl(Rl[2], "info", "52%", "03"),
			Jn("mesh-atlas", [Kn("mesh-atlas-icon", "texture", {
				tone: "success",
				size: 20
			}), Wn("mesh-atlas-copy", "shared texture atlas", { tone: "success" })], {
				gap: 9,
				align: "center",
				padding: [8, 10],
				frame: B("inset", { radius: 4 }),
				width: "fill"
			})
		], {
			gap: 9,
			padding: 16,
			frame: B("raised", { radius: 10 }),
			width: "fill"
		}), z("mesh-outputs", [Vl(Rl[3], "export", "GLB · GLTF · USDZ · NUCM", "success"), Vl(Rl[4], "camera", "PNG · GIF · VIDEO", "warning")], {
			gap: 12,
			justify: "center",
			width: "fill"
		})]
	}),
	edges: Hl,
	machine: zl.machine,
	controls: zl.controls,
	timeline: Xc([
		"mesh-stack",
		"mesh-portable",
		"mesh-pixels"
	], Hl.map((e) => e.id)),
	metadata: {
		source: "meshing-and-rendering/render-pipeline.svg",
		revision: 2
	}
}), "Focus any layer or output to inspect its rendering contract.", "The mesh assembles in draw order, then both output surfaces receive the completed geometry."), Wl = [
	{
		second: 0,
		active: 8
	},
	{
		second: 1,
		active: 21
	},
	{
		second: 2,
		active: 39
	},
	{
		second: 3,
		active: 62
	},
	{
		second: 4,
		active: 78
	},
	{
		second: 5,
		active: 86
	},
	{
		second: 6,
		active: 82
	},
	{
		second: 7,
		active: 88
	},
	{
		second: 8,
		active: 84
	},
	{
		second: 9,
		active: 87
	}
], Gl = Ue(.16, 1, .3, 1), Kl = We({
	frequency: 9.5,
	damping: 7.5
});
Nt({
	name: "paper-pastel",
	colors: {
		canvas: "#eeeae0",
		surface: "#f7f2e7",
		surfaceRaised: "#fffaf0",
		surfaceMuted: "#e9e2d4",
		text: "#292822",
		textMuted: "#716e63",
		accent: "#a16f93",
		accentContrast: "#fffaf0",
		info: "#789fc0",
		success: "#729d7b",
		warning: "#c39a55",
		danger: "#c77d77",
		connector: "#9c978a",
		border: "#d2cabc",
		chart1: "#6fae9e",
		chart2: "#9386b8",
		chart3: "#d18a73",
		chart4: "#c8aa60",
		chart5: "#c78496",
		chart6: "#7f9eba",
		chartPositive: "#729d7b",
		chartNegative: "#c77d77",
		chartNeutral: "#9c978a"
	},
	radii: {
		sm: 7,
		md: 12,
		lg: 18
	},
	typography: {
		body: {
			family: "\"Geist Mono\", ui-monospace, monospace",
			size: 14,
			lineHeight: 21,
			weight: 450
		},
		bodyStrong: {
			family: "\"Geist Mono\", ui-monospace, monospace",
			size: 15,
			lineHeight: 21,
			weight: 650
		},
		caption: {
			family: "\"Geist Mono\", ui-monospace, monospace",
			size: 12,
			lineHeight: 17,
			weight: 450
		},
		label: {
			family: "\"Geist Mono\", ui-monospace, monospace",
			size: 11,
			lineHeight: 15,
			weight: 650,
			letterSpacing: .65
		},
		title: {
			family: "\"Geist Mono\", ui-monospace, monospace",
			size: 22,
			lineHeight: 27,
			weight: 650,
			letterSpacing: -.4
		},
		display: {
			family: "\"Geist Mono\", ui-monospace, monospace",
			size: 36,
			lineHeight: 40,
			weight: 700,
			letterSpacing: -.8
		},
		code: {
			family: "\"Geist Mono\", ui-monospace, monospace",
			size: 13,
			lineHeight: 18,
			weight: 500
		}
	},
	motion: {
		fast: 140,
		normal: 300,
		slow: 680,
		easing: Gl
	},
	strokes: {
		hairline: 1,
		thin: 1.25,
		regular: 1.75,
		bold: 2.5
	},
	ornament: {
		grid: "lines",
		surface: "outlined",
		lineCap: "round",
		eyebrow: !0
	},
	materials: {
		raised: {
			fill: "surfaceRaised",
			stroke: "border",
			effects: [Dr({
				color: "text",
				opacity: .14,
				blur: 24,
				spread: 1,
				offset: [0, 10]
			}), jr({
				amount: .016,
				scale: .9,
				seed: 31
			})]
		},
		inset: {
			fill: "surfaceMuted",
			stroke: "border",
			effects: [Or({
				color: "text",
				opacity: .08,
				blur: 7,
				offset: [0, 2]
			})]
		}
	}
});
//#endregion
//#region ../scenes/dist/catalogue.js
var ql = [
	tl,
	ll,
	yl,
	Sl,
	El,
	Pl,
	zc,
	Ul,
	_c,
	{
		slug: "throughput-over-time",
		order: 91,
		title: "Plot in a card",
		summary: "A gradient area plot, live value, and summary measurements share one framed surface.",
		concept: "A plot is a composable scene fragment, not a special full-canvas widget.",
		interaction: "Inspect the line as a series or focus individual sampled dots.",
		animation: "The header arrives, the area and line draw together, then the measurements settle.",
		source: "Kineglyph quantitative example; all values are illustrative.",
		scene: br("throughput-over-time", {
			title: "Active chunks over time",
			description: "An illustrative stream trace sits inside a status card with a live value, operating band, target, and summary measurements.",
			metadata: {
				data: "illustrative",
				family: "quantitative",
				composition: "plot-in-card"
			}
		}, (e) => {
			let t = gc(Wl, {
				id: "stream-trend",
				x: "second",
				y: "active",
				marks: [
					cc({
						fill: ht("chart1", {
							from: .5,
							to: .015,
							angle: 90
						}),
						fillOpacity: 1,
						curve: "monotone"
					}),
					sc({
						tone: "chart1",
						curve: "monotone",
						interactive: "series"
					}),
					lc({
						tone: "chart1",
						pointRadius: 3,
						interactive: "marks"
					})
				],
				description: "Active chunks rise from 8 to the mid-eighties, then remain inside a 75-to-92 chunk operating band.",
				axes: {
					x: {
						label: "Elapsed time (s)",
						nice: !1,
						ticks: {
							wide: 7,
							compact: 5,
							narrow: 4
						}
					},
					y: {
						label: "Active chunks",
						domain: [0, 100],
						nice: !1
					}
				},
				annotations: [pc({
					y: [75, 92],
					tone: "success"
				}), fc({
					y: 80,
					tone: "success",
					dash: "dashed"
				})],
				grid: "y",
				legend: !1,
				height: {
					wide: 230,
					compact: 210,
					narrow: 180
				},
				motion: "auto",
				duration: 1350,
				easing: Gl
			}), n = e.add(t), r = e.stack([
				e.eyebrow("STREAM SAMPLE", {
					tone: "accent",
					id: "sample-label"
				}),
				e.title("Active chunks", { id: "sample-title" }),
				e.caption("One observation per second", { id: "sample-caption" })
			], {
				id: "sample-heading",
				gap: 3,
				grow: 1
			}), i = e.stack([e.title("87", {
				id: "current-value",
				align: "end"
			}), e.eyebrow("ACTIVE NOW", {
				id: "current-label",
				align: "end",
				tone: "success"
			})], {
				id: "current",
				gap: 2,
				width: 132,
				align: "end"
			}), a = e.row([r, i], {
				id: "sample-header",
				width: "fill",
				align: "end",
				justify: "between",
				gap: 24
			}), o = (t, n, r) => e.stack([e.heading(n, { id: `${t}-value` }), e.caption(r, { id: `${t}-label` })], {
				id: t,
				width: "fill",
				gap: 2,
				padding: [10, 12],
				frame: B("inset", {
					fill: "surfaceMuted",
					stroke: "border",
					radius: 6
				})
			}), s = o("average", "71.5", "mean active"), c = o("peak", "88", "peak active"), l = o("settled", "4 s", "to steady band"), u = o("target", "80", "target active"), d = e.grid([
				s,
				c,
				l,
				u
			], {
				id: "sample-stats",
				columns: {
					wide: 4,
					compact: 2
				},
				gap: 10,
				width: "fill"
			}), f = e.stack([
				a,
				n,
				d
			], {
				id: "stream-card",
				width: "fill",
				gap: {
					wide: 20,
					compact: 16,
					narrow: 14
				},
				padding: {
					wide: [24, 26],
					compact: [22, 22],
					narrow: [18, 16]
				},
				frame: B("raised", {
					fill: pt([
						{
							at: 0,
							color: "surfaceRaised"
						},
						{
							at: .58,
							color: "surface"
						},
						{
							at: 1,
							color: "surfaceMuted"
						}
					], { angle: 118 }),
					stroke: "border",
					radius: 12
				}),
				clip: !0
			});
			e.root(f), e.sequence([
				[e.reveal(r, {
					offset: 8,
					easing: Gl
				}), e.reveal(i, {
					offset: -8,
					easing: Gl
				})],
				e.reveal(n),
				e.reveal([
					s,
					c,
					l,
					u
				], {
					stagger: 90,
					offset: 6,
					scale: .97,
					easing: Kl
				})
			], { gap: 90 });
		})
	},
	Hc,
	Bc
].sort((e, t) => e.order - t.order);
function Jl(e) {
	return ql.find((t) => t.slug === e || t.scene.id === e);
}
var Yl = Nt({
	name: "nucleation-dark",
	colors: {
		canvas: "#101216",
		surface: "#16191e",
		surfaceRaised: "#1b1f25",
		surfaceMuted: "#13161a",
		text: "#e8eaed",
		textMuted: "#9299a3",
		accent: "#67cbbb",
		accentContrast: "#101216",
		info: "#7d8fd1",
		success: "#78c9a9",
		warning: "#dfbd79",
		danger: "#dc8c8c",
		connector: "#737b86",
		border: "#303640",
		chart1: "#67cbbb",
		chart2: "#8597d8",
		chart3: "#d59672",
		chart4: "#9fbd78",
		chart5: "#d58da2",
		chart6: "#aa9bd1",
		chartPositive: "#78c9a9",
		chartNegative: "#dc8c8c",
		chartNeutral: "#9299a3"
	},
	radii: {
		sm: 3,
		md: 6,
		lg: 8
	},
	typography: {
		body: {
			family: "Inter, \"Geist Sans\", ui-sans-serif, system-ui, sans-serif",
			size: 14,
			lineHeight: 21,
			weight: 450
		},
		bodyStrong: {
			family: "Inter, \"Geist Sans\", ui-sans-serif, system-ui, sans-serif",
			size: 15,
			lineHeight: 21,
			weight: 650
		},
		caption: {
			family: "Inter, \"Geist Sans\", ui-sans-serif, system-ui, sans-serif",
			size: 12,
			lineHeight: 17,
			weight: 450
		},
		label: {
			family: "\"Geist Mono\", ui-monospace, monospace",
			size: 10.5,
			lineHeight: 15,
			weight: 650,
			letterSpacing: .55
		},
		title: {
			family: "Inter, \"Geist Sans\", ui-sans-serif, system-ui, sans-serif",
			size: 22,
			lineHeight: 27,
			weight: 650,
			letterSpacing: -.35
		},
		display: {
			family: "Inter, \"Geist Sans\", ui-sans-serif, system-ui, sans-serif",
			size: 36,
			lineHeight: 40,
			weight: 700,
			letterSpacing: -.7
		},
		code: {
			family: "\"Geist Mono\", ui-monospace, monospace",
			size: 12.5,
			lineHeight: 18,
			weight: 500
		}
	},
	motion: {
		fast: 140,
		normal: 280,
		slow: 620,
		easing: "easeInOut"
	},
	strokes: {
		hairline: 1,
		thin: 1.15,
		regular: 1.5,
		bold: 2.25
	},
	ornament: {
		grid: "none",
		surface: "outlined",
		lineCap: "round",
		eyebrow: !0
	},
	materials: {
		flat: { fill: "canvas" },
		raised: {
			fill: "surfaceRaised",
			stroke: "border"
		},
		floating: {
			fill: "surfaceRaised",
			stroke: "border",
			effects: [Dr({
				color: "canvas",
				opacity: .22,
				blur: 12,
				offset: [0, 4]
			})]
		},
		inset: {
			fill: "surfaceMuted",
			stroke: "border"
		},
		glass: {
			fill: "surfaceRaised",
			stroke: "border"
		}
	}
}), Xl = {
	nucleation: Yl,
	"nucleation-dark": Yl,
	"nucleation-light": Nt({
		name: "nucleation-light",
		colors: {
			canvas: "#f4f1e9",
			surface: "#faf8f2",
			surfaceRaised: "#fffdf8",
			surfaceMuted: "#ece8de",
			text: "#25282d",
			textMuted: "#6e746f",
			accent: "#237f74",
			accentContrast: "#fffdf8",
			info: "#6475b7",
			success: "#4f9275",
			warning: "#a9792f",
			danger: "#b76060",
			connector: "#858b87",
			border: "#d4cfc4",
			chart1: "#4da99a",
			chart2: "#7f8fc7",
			chart3: "#c48765",
			chart4: "#91ad6c",
			chart5: "#c98297",
			chart6: "#9e90c0",
			chartPositive: "#4f9275",
			chartNegative: "#b76060",
			chartNeutral: "#858b87"
		}
	}, Yl),
	pock: Nt({
		name: "pock",
		colors: {
			canvas: "#060606",
			surface: "#0d0d0d",
			surfaceRaised: "#111612",
			surfaceMuted: "#0a0f0c",
			text: "#e6fff5",
			textMuted: "#83a397",
			accent: "#10b981",
			accentContrast: "#04120c",
			info: "#38bdf8",
			success: "#34d399",
			warning: "#fbbf24",
			danger: "#fb7185",
			connector: "#34d399",
			border: "#1b3329",
			chart1: "#10b981",
			chart2: "#38bdf8",
			chart3: "#fbbf24",
			chart4: "#a3e635",
			chart5: "#fb7185",
			chart6: "#a78bfa",
			chartPositive: "#34d399",
			chartNegative: "#fb7185",
			chartNeutral: "#83a397"
		},
		radii: {
			sm: 6,
			md: 12,
			lg: 18
		},
		typography: {
			body: {
				family: "\"Space Grotesk\", system-ui, sans-serif",
				size: 15,
				lineHeight: 22,
				weight: 450
			},
			bodyStrong: {
				family: "\"Space Grotesk\", system-ui, sans-serif",
				size: 16,
				lineHeight: 22,
				weight: 650
			},
			caption: {
				family: "\"Space Grotesk\", system-ui, sans-serif",
				size: 12.5,
				lineHeight: 18,
				weight: 450
			},
			label: {
				family: "\"JetBrains Mono\", ui-monospace, monospace",
				size: 10,
				lineHeight: 14,
				weight: 650,
				letterSpacing: 1
			},
			title: {
				family: "\"Space Grotesk\", system-ui, sans-serif",
				size: 24,
				lineHeight: 29,
				weight: 650,
				letterSpacing: -.5
			},
			display: {
				family: "\"Space Grotesk\", system-ui, sans-serif",
				size: 40,
				lineHeight: 44,
				weight: 700,
				letterSpacing: -1
			},
			code: {
				family: "\"JetBrains Mono\", ui-monospace, monospace",
				size: 13,
				lineHeight: 18,
				weight: 500
			}
		},
		motion: {
			fast: 120,
			normal: 260,
			slow: 560,
			easing: "easeOut"
		},
		strokes: {
			hairline: 1,
			thin: 1.5,
			regular: 2.25,
			bold: 3.5
		},
		ornament: {
			grid: "none",
			surface: "glow",
			lineCap: "round",
			eyebrow: !0
		}
	}),
	schematio: Nt({
		name: "schematio",
		colors: {
			canvas: "#202126",
			surface: "#2d2d2d",
			surfaceRaised: "#383a42",
			surfaceMuted: "#26272c",
			text: "#f7f8f8",
			textMuted: "#b6b9c3",
			accent: "#db45f0",
			accentContrast: "#ffffff",
			info: "#5ea0ff",
			success: "#a3f322",
			warning: "#ffba00",
			danger: "#ff647e",
			connector: "#e978fa",
			border: "#4a4d5a",
			chart1: "#db45f0",
			chart2: "#a3f322",
			chart3: "#5ea0ff",
			chart4: "#ffba00",
			chart5: "#c89cff",
			chart6: "#36d6c5",
			chartPositive: "#a3f322",
			chartNegative: "#ff647e",
			chartNeutral: "#b6b9c3"
		},
		radii: {
			sm: 8,
			md: 14,
			lg: 22
		},
		typography: {
			body: {
				family: "\"Figtree\", system-ui, sans-serif",
				size: 15,
				lineHeight: 22,
				weight: 450
			},
			bodyStrong: {
				family: "\"Figtree\", system-ui, sans-serif",
				size: 16,
				lineHeight: 22,
				weight: 650
			},
			caption: {
				family: "\"Figtree\", system-ui, sans-serif",
				size: 12.5,
				lineHeight: 18,
				weight: 450
			},
			label: {
				family: "\"Figtree\", system-ui, sans-serif",
				size: 11,
				lineHeight: 15,
				weight: 700,
				letterSpacing: .75
			},
			title: {
				family: "\"Figtree\", system-ui, sans-serif",
				size: 25,
				lineHeight: 30,
				weight: 700,
				letterSpacing: -.5
			},
			display: {
				family: "\"Figtree\", system-ui, sans-serif",
				size: 42,
				lineHeight: 46,
				weight: 700,
				letterSpacing: -1
			},
			code: {
				family: "ui-monospace, SFMono-Regular, Menlo, monospace",
				size: 13,
				lineHeight: 18,
				weight: 500
			}
		},
		motion: {
			fast: 150,
			normal: 300,
			slow: 650,
			easing: "easeInOut"
		},
		strokes: {
			hairline: 1,
			thin: 1.5,
			regular: 2,
			bold: 3
		},
		ornament: {
			grid: "lines",
			surface: "flat",
			lineCap: "round",
			eyebrow: !1
		}
	})
}, Zl = [
	"nucleation",
	"pock",
	"schematio"
], Ql = {
	nucleation: {
		label: "Nucleation",
		note: "Basalt / Vellum · quiet, technical, editorial"
	},
	pock: {
		label: "Pock",
		note: "Black / emerald · secure, luminous, kinetic"
	},
	schematio: {
		label: "Schematio",
		note: "Graphite / fuchsia · soft, spatial, product-led"
	}
};
function $l(e) {
	return e === "nucleation" || e === "pock" || e === "schematio";
}
//#endregion
//#region src/bundle.ts
for (let e of ql) Eo(e.slug, e.scene);
for (let [e, t] of Object.entries(Xl)) Do(e, t);
//#endregion
export { uo as FIGURE_STYLES, po as LiveSurfaceManager, lo as STYLE_ID, ht as alphaGradient, cc as area, Oo as autoMount, Ar as backdrop, ic as bar, kr as blur, mc as calloutAt, ql as catalogue, Nn as createMachineState, Nt as createTheme, Mt as defaultTheme, Ct as defineScene, lc as dot, fo as ensureStyles, br as figure, Jl as findCatalogueEntry, ac as groupedBar, dc as heatmap, Or as innerShadow, $l as isThemeName, sc as line, pt as linearGradient, B as material, vo as modelViewerSurface, xo as mountKineglyph, jr as noise, gc as plot, hc as pointLabel, mt as radialGradient, pc as range, Eo as registerScene, Do as registerTheme, Si as resolveFigure, fc as rule, Mr as shader, Dr as shadow, uc as sparkline, oc as stackedBar, ko as startWhenVisible, Ql as themeCopy, Zl as themeNames, Xl as themes };

//# sourceMappingURL=kineglyph-web.js.map