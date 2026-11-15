const $ = (selector, root = document) => root.querySelector(selector);
const $$ = (selector, root = document) => [...root.querySelectorAll(selector)];
const nf = new Intl.NumberFormat();
const PAGE_SIZE = 60;
const sourceNames = {
  'logical-redstone-archive': 'Logical Redstone', 'slimestone-military': 'SMA · Slimestone',
  'slimestone-ai': 'Slimestone Research', 'storage-catalog': 'Storage Catalog',
  'soontech-current': 'Soontech · Current', 'soontech-archive': 'Soontech · Archive',
  'autocrafting': 'Autocrafting', 'casual-builds': 'Casual Minecraft',
  'wither-archive': 'Wither Archive', 'lithium-gametests': 'Lithium GameTests',
  'redstonepedia': 'Redstonepedia', 'curated-candidates': 'Curated Mechanisms',
  'user-world-record-plot': 'World Record Plot', 'world-record-parts': 'World Record · Extracted builds',
  'world-record-floor-parts': 'World Record · Floor attachments',
};
const state = {
  groups: [], byId: new Map(), summary: {}, collection: 'all', source: null,
  query: '', filter: 'all', sort: 'recommended', view: 'grid', limit: PAGE_SIZE,
  selected: null, detail: null, property: 'overview', viewer: 'inspect',
  entryAbort: null, captureAbort: null, replayCleanup: null, liveFrame: null,
  favorites: readFavorites(), opener: null, toastTimer: null,
};

function el(tag, className, text) {
  const node = document.createElement(tag);
  if (className) node.className = className;
  if (text !== undefined && text !== null) node.textContent = String(text);
  return node;
}
function button(text, className, action) {
  const node = el('button', className, text); node.type = 'button';
  if (action) node.addEventListener('click', action);
  return node;
}
function count(value) { return Number.isFinite(Number(value)) && value !== null && value !== '' ? nf.format(Number(value)) : 'Not recorded'; }
function size(value) {
  if (!Number.isFinite(Number(value)) || value === null) return 'Not recorded';
  const n = Number(value); return n < 1024 ? `${n} B` : n < 1048576 ? `${(n / 1024).toFixed(1)} KB` : `${(n / 1048576).toFixed(2)} MB`;
}
function sourceName(id) { return sourceNames[id] || String(id || 'Unknown source').replaceAll('-', ' '); }
function title(row) { return row.title || row.name || String(row.path || 'Untitled schematic').split('/').pop().replace(/\.[^.]+$/, '').replaceAll('_', ' '); }
function hash(row) { return row.sha256 || row.id; }
function isPlotPart(row) { return ['world-record-parts', 'world-record-floor-parts'].includes(row.source); }
function isPlotDoor(row) { return isPlotPart(row) && row.classification === 'door_candidate' && !(row.review_tags || []).includes('infrastructure_review'); }
function isPlot(row) { return row.source === 'user-world-record-plot' || (!isPlotPart(row) && /WorldRecordPlot/i.test(row.path || '')); }
function blocks(row) { return row.declared_blocks ?? row.non_air_blocks ?? row.static?.non_air_blocks ?? row.static?.non_air_region_sum ?? row.saved_metadata?.TotalBlocks ?? row.counts?.non_air_blocks ?? null; }
function dimensions(row) {
  const d = row.dimensions || row.static?.dimensions_xyz || row.saved_metadata?.EnclosingSize || row.region?.dimensions;
  if (Array.isArray(d)) return d;
  if (d && ['x', 'y', 'z'].every(a => d[a] !== undefined)) return [d.x, d.y, d.z];
  const bounds = row.block_bounds || row.bounds;
  if (bounds?.min?.length === 3 && bounds?.max?.length === 3) return bounds.max.map((v, i) => v - bounds.min[i] + 1);
  return null;
}
function dimensionLabel(row) { return dimensions(row)?.map(v => count(v)).join(' × ') || 'Not recorded'; }
function textValue(value) {
  if (value === undefined || value === null) return 'Not recorded';
  if (typeof value === 'boolean') return value ? 'Yes' : 'No';
  if (typeof value === 'object') return JSON.stringify(value);
  return String(value);
}
function readable(value) { return String(value).replace(/^minecraft:/, '').replaceAll('_', ' '); }
function readFavorites() {
  try { const values = JSON.parse(localStorage.getItem('nucleation-atlas-favorites') || '[]'); return new Set(Array.isArray(values) ? values.filter(x => typeof x === 'string') : []); }
  catch { return new Set(); }
}
function toast(message) {
  clearTimeout(state.toastTimer); $('#toast').textContent = message; $('#toast').hidden = false;
  state.toastTimer = setTimeout(() => { $('#toast').hidden = true; }, 2600);
}
function toggleFavorite(id) {
  if (state.favorites.has(id)) state.favorites.delete(id); else state.favorites.add(id);
  try { localStorage.setItem('nucleation-atlas-favorites', JSON.stringify([...state.favorites])); }
  catch { toast('Saved for this visit; browser storage is unavailable.'); }
  renderNav(); renderResults();
  if (state.selected === id) renderDetailBadges();
}
function validation(row) {
  const raw = String(row.validation_status || '').toLowerCase();
  if (/unsupported/.test(raw)) return { key: 'review', label: 'Engine unsupported' };
  if (/vanilla.*(parity|match).*pass|vanilla_tick_end_parity_pass/.test(raw)) return { key: 'vanilla', label: 'Vanilla matched' };
  if (/functional_fail|failed|mismatch/.test(raw)) return { key: 'fail', label: 'Check failed' };
  if (/functional_pass/.test(raw)) return { key: 'functional', label: 'Functional checks' };
  if (row.readiness === 'known-negative') return { key: 'negative', label: 'Known negative' };
  if (row.readiness === 'quarantine') return { key: 'review', label: 'Needs review' };
  return { key: 'unvalidated', label: 'Unvalidated' };
}
function chip(row) {
  const v = validation(row); const node = el('span', `chip ${v.key}`, v.label);
  node.title = row.validation_detail || (v.key === 'functional' ? 'Engine functional checks; no implied vanilla parity.' : v.key === 'negative' ? 'Source-labeled negative; not a new observed failure.' : 'No passing validation evidence recorded.');
  return node;
}
function relevance(row) {
  const score = { vanilla: 500, functional: 400, unvalidated: 100, review: 20, fail: 10, negative: 30 }[validation(row).key];
  return score + (row.priority === 'high' ? 90 : row.priority === 'medium' ? 40 : 0) + (isPlotDoor(row) ? 160 : 0);
}
function evidenceText(row) {
  if (row.validation_detail) return textValue(row.validation_detail);
  if (isPlotDoor(row)) return 'This is a physical door candidate extracted from the saved plot. Its structural properties are recorded; activation, complete cycles and vanilla parity still need functional checks.';
  if (isPlotPart(row)) return 'This plot fragment has been isolated geometrically. It may be an incomplete mechanism, supporting structure or several connected builds. No passing functional check is recorded.';
  if (row.readiness === 'known-negative') return 'The source labels this as a negative case. That label is useful for test selection; it is not a newly measured outcome.';
  if (row.readiness === 'quarantine') return 'This candidate needs review before it can support a functional or performance claim.';
  return 'No passing functional or vanilla comparison is recorded for this file. A successful download or static inspection does not prove operation.';
}
async function getJSON(url, signal) {
  const response = await fetch(url, { signal, headers: { Accept: 'application/json' } });
  if (!response.ok) throw new Error(`Request failed (${response.status})`);
  return response.json();
}

function buildGroups(rows) {
  const groups = new Map();
  for (const row of rows) {
    const id = hash(row); if (!id) continue;
    if (!groups.has(id)) groups.set(id, { id, rows: [] });
    groups.get(id).rows.push(row);
  }
  for (const group of groups.values()) {
    group.rows.sort((a, b) => relevance(b) - relevance(a)); group.row = group.rows[0];
    group.search = group.rows.map(row => [title(row), row.source, sourceName(row.source), row.path, row.category, row.classification, row.data_version, ...(row.tags || []), ...(row.review_tags || []), ...(row.declared_versions || [])].join(' ')).join(' ').toLowerCase();
  }
  state.groups = [...groups.values()]; state.byId = groups;
}
const collections = [
  ['all', 'All schematics', '▦'], ['plot-doors', 'Plot doors', '▥'],
  ['plot-fragments', 'Plot fragments', '⊡'], ['logic', 'Logic & computation', '⌘'],
  ['movement', 'Pistons & movement', '↗'], ['storage', 'Items & storage', '▤'],
  ['layouts', 'Research layouts', '⊞'], ['favorites', 'Saved for later', '☆'],
];
function belongs(group, collection) {
  const row = group.row;
  if (collection === 'favorites') return state.favorites.has(group.id);
  if (collection === 'plot-doors') return group.rows.some(isPlotDoor);
  if (collection === 'plot-fragments') return group.rows.some(row => isPlotPart(row) && !isPlotDoor(row));
  if (collection === 'layouts') return row.kind === 'layout';
  if (row.kind === 'layout') return false;
  if (collection === 'all') return true;
  if (collection === 'logic') return /logical-redstone|logic|adder|counter|decoder|latch|memory|computation|comparator/.test(group.search);
  if (collection === 'movement') return /piston|flying|slime|movement|stepper|door|worldrecordplot/.test(group.search);
  if (collection === 'storage') return /storage|inventory|item|hopper|shulker|crafter|smelter|sorter|autocraft/.test(group.search);
  return false;
}
function chooseCollection(collection, source = null) {
  state.collection = collection; state.source = source; state.limit = PAGE_SIZE;
  renderNav(); renderResults();
  history.replaceState(null, '', collectionURL());
}
function collectionURL() {
  const params = new URLSearchParams();
  if (state.source) params.set('source', state.source);
  else if (state.collection !== 'all') params.set('collection', state.collection);
  return location.pathname + location.search + (params.size ? '#' + params : '');
}
function renderNav() {
  $('#collections').replaceChildren();
  for (const [id, label, symbol] of collections) {
    const node = button('', `nav-item${state.collection === id && !state.source ? ' active' : ''}`, () => chooseCollection(id));
    node.append(el('span', 'nav-icon', symbol), el('span', 'nav-name', label), el('span', 'nav-count', nf.format(state.groups.filter(g => belongs(g, id)).length)));
    if (state.collection === id && !state.source) node.setAttribute('aria-current', 'page');
    $('#collections').append(node);
  }
  const sources = new Map();
  for (const group of state.groups) for (const id of new Set(group.rows.map(r => r.source))) sources.set(id, (sources.get(id) || 0) + 1);
  $('#source-count').textContent = sources.size; $('#sources').replaceChildren();
  for (const [id, total] of [...sources.entries()].sort((a, b) => sourceName(a[0]).localeCompare(sourceName(b[0])))) {
    const node = button('', `nav-item${state.source === id ? ' active' : ''}`, () => chooseCollection('all', id));
    node.append(el('span', 'nav-icon', '◈'), el('span', 'nav-name', sourceName(id)), el('span', 'nav-count', nf.format(total)));
    if (state.source === id) node.setAttribute('aria-current', 'page');
    $('#sources').append(node);
  }
}
function renderSummary() {
  const s = state.summary;
  const values = [[s.schematic_files, 'schematic files'], [s.unique_schematics ?? s.unique_schematic_contents, 'unique schematics'], [s.sources, 'source collections'], [s.validated, 'with passing checks']];
  const stats = $('#summary-stats'); stats.replaceChildren();
  for (const [value, label] of values) { const n = el('div', 'summary-stat'); n.append(el('strong', '', count(value)), el('span', '', label)); stats.append(n); }
}
function thumbnail(row, large = false) {
  const art = el('div', `${large ? 'overview-art' : 'card-art dark-art'} native-thumbnail`);
  const preview = row.thumbnail || state.byId.get(hash(row))?.row?.thumbnail || {};
  const pending = el('div', 'thumbnail-state');
  const label = el('span', 'thumbnail-state-label');
  const message = el('strong');
  const explanation = el('span', 'thumbnail-state-description');
  pending.append(label, message, explanation); art.append(pending);
  function showMissing(failed = false) {
    const invisible = failed && /mesher produced no triangles/.test(preview.error || '');
    art.dataset.previewStatus = failed ? 'error' : 'missing';
    label.textContent = row.kind === 'layout' ? 'RESEARCH LAYOUT' : 'SAVED FILE PREVIEW';
    message.textContent = invisible ? 'No visible geometry' : failed ? 'Render unavailable' : row.kind === 'layout' ? 'Text layout' : isPlot(row) ? 'Explore the extracted doors' : 'Preview not generated yet';
    explanation.textContent = invisible ? 'Saved blocks such as barriers are invisible. Inspect the materials and bounds.' : row.kind === 'layout' ? 'Open the file to inspect its layout.' : isPlot(row) ? 'Individual builds are in the Plot doors collection.' : failed ? 'The original file and its properties are still available.' : 'A textured render will appear here when available.';
    if (failed && preview.error) pending.title = textValue(preview.error);
  }
  if (preview.status === 'error' || preview.status === 'missing' || row.kind === 'layout') {
    showMissing(preview.status === 'error');
  } else {
    art.dataset.previewStatus = 'loading'; label.textContent = 'NATIVE 3D RENDER'; message.textContent = 'Loading preview…';
    const image = document.createElement('img'); image.loading = large ? 'eager' : 'lazy'; image.decoding = 'async'; image.style.opacity = '0';
    image.width = 768; image.height = 576;
    image.alt = `Textured isometric render of ${title(row)}, as saved; no simulation ticks applied`;
    image.addEventListener('load', () => { image.style.opacity = ''; pending.remove(); art.dataset.previewStatus = 'ok'; }, { once: true });
    image.addEventListener('error', () => { image.remove(); showMissing(); }, { once: true });
    image.src = `/api/thumbnail/${encodeURIComponent(hash(row))}`; art.append(image);
  }
  if (large && isPlot(row)) { const link = el('a', 'map-open', 'Open annotated plot map ↗'); link.href = '/api/plot-map'; link.target = '_blank'; link.rel = 'noopener'; art.append(link); }
  return art;
}
function renderFeatured() {
  const plot = state.groups.find(g => g.rows.some(isPlot)); const container = $('#featured');
  const doors = state.groups.filter(g => g.rows.some(isPlotDoor));
  if (!plot && !doors.length) { container.hidden = true; return; }
  container.hidden = false; container.replaceChildren();
  const copy = el('div', 'featured-copy'); copy.append(el('p', 'eyebrow', 'WORLD RECORD PLOT / EXTRACTED COLLECTION'), el('h2', '', 'Doors, down to the details.'), el('p', '', doors.length ? `${count(doors.length)} physical door candidates, isolated from the saved plot. Inspect the builds, their bounds and the checks still needed.` : 'Individual door candidates are being isolated from the saved plot. Browse their renders and extraction notes as they become available.'));
  const actions = el('div', 'feature-actions');
  actions.append(button('Browse extracted doors  ↗', 'feature-link', () => { chooseCollection('plot-doors'); $('#library-title').scrollIntoView({ block: 'start', behavior: 'smooth' }); }));
  if (plot) actions.append(button('Original plot & map', 'feature-secondary', event => navigate(plot.id, event.currentTarget)));
  copy.append(actions);
  const featured = doors.find(g => g.row.thumbnail?.status === 'ok' && blocks(g.row) >= 200 && blocks(g.row) <= 3000 && Math.max(...(dimensions(g.row) || [Infinity])) <= 32 && !(g.row.review_tags || []).some(tag => ['plot_boundary', 'no_saved_activation_control'].includes(tag))) || doors.find(g => g.row.thumbnail?.status === 'ok') || doors[0];
  const art = el('div', 'featured-art');
  if (featured) {
    const open = button('', 'featured-preview', event => navigate(featured.id, event.currentTarget));
    open.setAttribute('aria-label', `Inspect ${title(featured.row)}`); open.append(thumbnail(featured.row)); art.append(open);
  } else art.append(el('p', 'feature-pending', 'Extracted door previews will appear here.'));
  container.append(copy, art);
}
function matchingGroups() {
  const tokens = state.query.trim().toLowerCase().split(/\s+/).filter(Boolean);
  const result = state.groups.filter(group => {
    if (state.source ? !group.rows.some(r => r.source === state.source) : !belongs(group, state.collection)) return false;
    if (tokens.some(token => !group.search.includes(token))) return false;
    const v = validation(group.row).key;
    return state.filter === 'all' || state.filter === v || (state.filter === 'review' && ['review', 'fail', 'negative'].includes(v));
  });
  result.sort((a, b) => {
    if (state.sort === 'name') return title(a.row).localeCompare(title(b.row));
    if (state.sort === 'smallest') return (blocks(a.row) ?? Infinity) - (blocks(b.row) ?? Infinity) || title(a.row).localeCompare(title(b.row));
    if (state.sort === 'largest') return (blocks(b.row) ?? -1) - (blocks(a.row) ?? -1) || title(a.row).localeCompare(title(b.row));
    return relevance(b.row) - relevance(a.row) || title(a.row).localeCompare(title(b.row));
  }); return result;
}
function favoriteButton(id, table = false) {
  const saved = state.favorites.has(id); const node = button(saved ? '★' : '☆', `${table ? 'table-favorite' : 'favorite-button'}${saved ? ' saved' : ''}`, () => toggleFavorite(id));
  node.setAttribute('aria-label', saved ? 'Remove from saved schematics' : 'Save schematic'); node.setAttribute('aria-pressed', String(saved)); return node;
}
function card(group) {
  const row = group.row; const node = el('article', 'schematic-card');
  const open = button('', 'card-open', event => navigate(group.id, event.currentTarget)); open.setAttribute('aria-label', `Inspect ${title(row)}`);
  open.append(thumbnail(row)); const body = el('div', 'card-body');
  body.append(el('p', 'card-eyebrow', sourceName(row.source) + (group.rows.length > 1 ? ` · ${group.rows.length} sources / paths` : '')), el('h3', 'card-title', title(row)));
  const meta = el('div', 'card-meta'); meta.append(el('span', '', blocks(row) === null ? 'Blocks unknown' : `${count(blocks(row))} blocks`), el('span', '', row.data_version == null ? 'DV unknown' : `DV ${row.data_version}`)); body.append(meta);
  const footer = el('div', 'card-footer'); footer.append(chip(row), el('span', 'card-format', row.kind === 'layout' ? 'TEXT LAYOUT' : String(row.path).split('.').pop())); body.append(footer); open.append(body);
  node.append(open, favoriteButton(group.id)); return node;
}
function table(groups) {
  const table = el('table'); const head = el('thead'); const tr = el('tr');
  for (const name of ['Mechanism', 'Evidence', 'Blocks', 'Data version', 'File size', 'Saved']) tr.append(el('th', '', name)); head.append(tr); table.append(head);
  const body = el('tbody'); for (const group of groups) {
    const row = group.row; const tr = el('tr'); const name = el('td'); const open = button(title(row), 'table-title', event => navigate(group.id, event.currentTarget)); name.append(open, el('span', 'table-source', sourceName(row.source))); tr.append(name);
    const evidence = el('td'); evidence.append(chip(row)); tr.append(evidence, el('td', 'table-number', blocks(row) === null ? '—' : count(blocks(row))), el('td', 'table-number', row.data_version ?? 'Unknown'), el('td', 'table-number', size(row.bytes)));
    const fav = el('td'); fav.append(favoriteButton(group.id, true)); tr.append(fav); body.append(tr);
  } table.append(body); return table;
}
function renderResults() {
  const result = matchingGroups(); const shown = result.slice(0, state.limit);
  const heading = state.source ? sourceName(state.source) : collections.find(c => c[0] === state.collection)?.[1] || 'All schematics';
  $('#library-title').textContent = heading; $('#current-collection').textContent = heading;
  $('#result-count').textContent = `${nf.format(result.length)} unique files`;
  const container = $('#results'); container.setAttribute('aria-busy', 'false'); container.replaceChildren(); container.className = state.view === 'grid' ? 'card-grid' : 'results-table';
  if (!result.length) {
    container.className = 'card-grid'; const empty = el('div', 'empty-state'); empty.append(el('span', 'nav-icon', '⌕'), el('h3', '', state.collection === 'favorites' ? 'Make this collection yours.' : 'No matching mechanisms'), el('p', '', state.collection === 'favorites' ? 'Use the star on a schematic to keep it here for later.' : 'Try a broader name, mechanism, source, or native data version.'));
    empty.append(button('Reset filters', 'button secondary', resetFilters)); container.append(empty);
  } else if (state.view === 'grid') container.append(...shown.map(card)); else container.append(table(shown));
  $('#page-count').textContent = result.length ? `Showing ${nf.format(shown.length)} of ${nf.format(result.length)} · grouped by identical file contents` : '';
  $('#load-more').hidden = shown.length >= result.length;
  $('#active-filter').hidden = !state.query && state.filter === 'all'; $('#active-filter').replaceChildren();
  if (!$('#active-filter').hidden) { $('#active-filter').append(el('span', '', [state.query && `“${state.query}”`, state.filter !== 'all' && $('#validation-filter').selectedOptions[0].text].filter(Boolean).join(' · ')), button('Clear', '', resetFilters)); }
}
function resetFilters() { state.query = ''; state.filter = 'all'; state.limit = PAGE_SIZE; $('#search').value = ''; $('#validation-filter').value = 'all'; renderResults(); }
function navigate(id, opener) { state.opener = opener || document.activeElement; if (location.hash === `#entry=${id}`) openDetail(id); else location.hash = `entry=${encodeURIComponent(id)}`; }
function handleHash() {
  const params = new URLSearchParams(location.hash.slice(1)); const id = params.get('entry');
  if (id) { openDetail(id); return; }
  if ($('#detail').open) $('#detail').close();
  const collection = params.get('collection'), source = params.get('source');
  if (source && state.groups.some(g => g.rows.some(r => r.source === source))) chooseCollection('all', source);
  else if (collections.some(c => c[0] === collection)) chooseCollection(collection);
}
function primaryDetail() { return state.detail?.entries?.[0] || state.byId.get(state.selected)?.row || {}; }
function clearReplay() {
  state.captureAbort?.abort(); state.captureAbort = null;
  try { if (typeof state.replayCleanup === 'function') state.replayCleanup(); else state.replayCleanup?.dispose?.(); } catch { /* A stale viewer must not block changing files. */ }
  state.replayCleanup = null; $('#replay-stage').replaceChildren();
}
function clearLive() {
  state.liveFrame?.remove(); state.liveFrame = null; $('#live-stage').replaceChildren();
  for (const id of ['live-tick', 'live-tps', 'live-entities']) $(`#${id}`).textContent = '—';
  $('#live-status').textContent = 'Not started'; $('#engine-extra').textContent = '';
}
async function openDetail(id) {
  if (state.selected === id && $('#detail').open) return;
  if (!/^[a-f0-9]{64}$/i.test(id)) { toast('This file identifier is not valid.'); return; }
  state.entryAbort?.abort(); clearReplay(); clearLive();
  state.selected = id; state.detail = null; state.property = 'overview';
  const row = state.byId.get(id)?.row || { sha256: id, title: 'Schematic details' };
  $('#detail-title').textContent = title(row); $('#detail-source').textContent = sourceName(row.source); $('#detail-loading').hidden = false;
  $('#download-file').href = `/api/file/${encodeURIComponent(id)}`; $('#launch-live').disabled = true;
  $('#launch-note').textContent = 'Checking the available engine and capture capabilities…'; $('#launch-note').className = 'launch-note';
  $('#capture-count').textContent = ''; $('#capture-select').replaceChildren();
  renderDetailBadges(); renderProperties(); renderInspect(row); setViewer('inspect');
  if (!$('#detail').open) $('#detail').showModal();
  document.body.style.overflow = 'hidden';
  const controller = new AbortController(); state.entryAbort = controller;
  try {
    const result = await getJSON(`/api/entry/${encodeURIComponent(id)}`, controller.signal);
    if (state.selected !== id) return;
    result.entries = (result.entries || []).sort((a, b) => relevance(b) - relevance(a)); state.detail = result;
    const current = primaryDetail(); $('#detail-title').textContent = title(current); $('#detail-source').textContent = `${sourceName(current.source)} / ${current.kind === 'layout' ? 'RESEARCH LAYOUT' : 'SCHEMATIC'}`;
    renderDetailBadges(); renderProperties(); renderInspect(current); configureCaptures(); configureLive();
    if (state.viewer === 'replay') loadCapture();
  } catch (error) {
    if (error.name !== 'AbortError') { $('#launch-note').textContent = 'Details could not be loaded. The original file and listed metadata remain available.'; $('#property-content').append(el('p', 'detail-note', error.message)); }
  } finally { if (state.selected === id) $('#detail-loading').hidden = true; }
}
function renderDetailBadges() {
  const row = primaryDetail(); $('#detail-badges').replaceChildren(chip(row));
  if (row.kind === 'layout') $('#detail-badges').append(el('span', 'tag', 'Text layout'));
  if (isPlotPart(row)) $('#detail-badges').append(el('span', 'tag', isPlotDoor(row) ? 'Physical door candidate' : 'Plot fragment'));
  const saved = state.favorites.has(state.selected); const fav = button(saved ? '★ Saved' : '☆ Save for later', `favorite-detail${saved ? ' saved' : ''}`, () => toggleFavorite(state.selected)); fav.setAttribute('aria-pressed', String(saved)); $('#detail-badges').append(fav);
}
function renderInspect(row) { $('#pane-inspect').replaceChildren(thumbnail({ ...row, sha256: state.selected }, true)); }
function sectionTitle(text) { return el('h3', 'property-title', text); }
function facts(items) {
  const list = el('dl', 'facts'); for (const [name, value] of items) { const item = el('div', 'fact'); item.append(el('dt', '', name), el('dd', '', value)); list.append(item); } return list;
}
function listNotes(values) {
  const list = el('ul', 'notes-list');
  for (const value of values) { if (value === null || value === undefined) continue; list.append(el('li', '', textValue(value))); }
  return list;
}
function safeExternal(url) { try { const parsed = new URL(url, location.href); return ['https:', 'http:'].includes(parsed.protocol) ? parsed.href : null; } catch { return null; } }
function renderExtraction(content, row) {
  const bounds = row.block_bounds || row.bounds || {};
  const contacts = row.cutoff_face_contacts || {};
  const contactLabels = Array.isArray(contacts) ? contacts.map(textValue) : Object.entries(contacts).filter(([, n]) => n).map(([face, n]) => `${readable(face)} · ${count(n)} blocks`);
  content.append(sectionTitle('Isolated from the World Record Plot'), facts([
    ['Classification', readable(row.classification || 'unclassified')],
    ['Pistons', count(row.pistons?.total)],
    ['Saved controls', count(row.controls?.count)],
    ['Saved inventory entries', count(row.inventory?.saved_entries)],
    ['Source minimum X / Y / Z', textValue(bounds.min)],
    ['Source maximum X / Y / Z', textValue(bounds.max)],
  ]));
  content.append(el('p', 'detail-note extraction-note', 'Bounds use the original selection coordinates. Physical isolation does not establish a complete door: detached parts can be separate, and connected builds can share a component.'));
  if (row.restored_platform_context) {
    const context = row.restored_platform_context;
    content.append(sectionTitle('Shared-floor separation'), facts([
      ['Parent component', row.parent_part || 'Not recorded'],
      ['Platform height', count(context.floor_y)],
      ['Original floor cells restored', count(context.count)],
      ['Horizontal margin', `${count(context.horizontal_halo)} block`],
    ]), el('p', 'detail-note extraction-note', 'Shared platform cells were excluded from grouping, then restored from the source around this build. Original holes remain. Floor context may overlap another export; movement beyond this footprint still needs review.'));
  }
  if (contactLabels.length) content.append(sectionTitle('Original selection edges'), el('p', 'evidence-box review', `Touches ${contactLabels.join('; ')}. The original selection may have cut off part of this build.`));
  const reviewTags = row.review_tags || [];
  if (reviewTags.length) {
    const tags = el('div', 'tag-list review-tags');
    for (const tag of reviewTags) tags.append(el('span', 'tag', readable(tag)));
    content.append(sectionTitle('Needs review'), tags);
  }
  const inventory = row.inventory;
  if (inventory) content.append(sectionTitle('Saved inventory evidence'), facts([
    ['Containers', count(row.containers?.count)], ['Missing block entities', count(row.containers?.missing_block_entities)],
    ['Nonempty inventory tags', count(inventory.saved_nonempty)], ['Empty inventory tags', count(inventory.saved_empty)],
    ['Absent inventory tags', count(inventory.tag_absent)], ['Saved item count', count(inventory.saved_item_count)],
  ]), el('p', 'detail-note extraction-note', 'Empty or absent saved inventory data may omit items the original mechanism needs. The render shows saved geometry; it does not repair the circuit.'));
}
function renderProperties() {
  $$('.property-tabs button').forEach(node => { const active = node.dataset.property === state.property; node.classList.toggle('active', active); node.setAttribute('aria-selected', String(active)); node.tabIndex = active ? 0 : -1; });
  const content = $('#property-content'); content.replaceChildren(); content.setAttribute('aria-labelledby', `property-tab-${state.property}`); const row = primaryDetail(); const inspected = state.detail?.inspection || {};
  if (state.property === 'overview') {
    const savedDimensions = inspected.dimensions;
    const dimensionsText = Array.isArray(savedDimensions) ? savedDimensions.map(count).join(' × ') : dimensionLabel(row);
    content.append(sectionTitle('Saved file'), facts([['Placed blocks', count(inspected.non_air_blocks ?? blocks(row))], ['Dimensions · X × Y × Z', dimensionsText], ['Native data version', inspected.native_data_version ?? row.data_version ?? row.static?.native_data_version ?? row.native_data_version ?? 'Unknown'], ['File size', size(row.bytes)], ['Saved entities', count(inspected.entities ?? row.entities ?? row.static?.entities ?? row.static?.entity_count)], ['Block entities', count(inspected.block_entities ?? row.block_entities ?? row.static?.block_entities ?? row.static?.block_entity_count)]]));
    content.append(sectionTitle('Validation evidence'), el('p', `evidence-box ${validation(row).key}`, evidenceText(row)));
    if (isPlotPart(row)) renderExtraction(content, row);
    if (inspected.scope || inspected.note) content.append(el('p', 'detail-note', inspected.scope || inspected.note));
    const tags = el('div', 'tag-list'); for (const text of (row.tags || []).slice(0,40)) tags.append(el('span', 'tag', text));
    if (tags.children.length) content.append(sectionTitle('Mechanisms & labels'), tags);
    if (inspected.bounds) content.append(sectionTitle('Saved bounds'), facts([['Minimum X / Y / Z', textValue(inspected.bounds.min)], ['Maximum X / Y / Z', textValue(inspected.bounds.max)]]));
    const controls = inspected.saved_controls || row.control_positions || row.controls?.locations || [];
    if (Array.isArray(controls) && controls.length) {
      content.append(sectionTitle(`Saved controls · ${count(controls.length)}`), el('p', 'detail-note', 'Saved controls are not yet an activation protocol. Use the recorded test instructions where available.'));
      content.append(listNotes(controls.slice(0,20).map(c => `${textValue(c.pos || c.position)} · ${c.state || c.block || c.type || 'control'}`)));
    }
    const candidates = row.candidates || row.curation?.candidates || [];
    if (isPlot(row)) {
      content.append(sectionTitle('Extracted plot doors'), el('p', 'detail-note', 'Browse the individual physical candidates to inspect rendered builds, saved controls and possible cutoffs. Their functional behaviour still needs checking.'), button('Browse extracted doors ↗', 'button secondary', () => { closeDetails(); chooseCollection('plot-doors'); }));
      content.append(sectionTitle('Original survey'), el('p', 'detail-note', 'The original map and survey bounds provide context for the extracted collection. Survey bounds alone do not prove a complete circuit.'));
      if (Array.isArray(candidates) && candidates.length) {
        for (const c of candidates.slice(0,20)) {
          const item = el('div', 'source-card'); item.append(el('h3', '', c.name || c.title || c.id || c.candidate_id || 'Candidate'), el('code', '', textValue(c.bounds || c.bounding_box || c.geometry?.bounds || 'Bounds not recorded')));
          if (c.rationale || c.reason) item.append(el('p', 'detail-note', textValue(c.rationale || c.reason))); content.append(item);
        }
      } else { const link = el('a', 'button secondary', 'View annotated plot map ↗'); link.href = '/api/plot-map'; link.target = '_blank'; link.rel = 'noopener'; content.append(link); }
    }
  } else if (state.property === 'materials') {
    const materials = inspected.materials || row.static?.material_counts || row.block_counts || {};
    const rows = Object.entries(materials).filter(([, v]) => typeof v === 'number').sort((a, b) => b[1] - a[1]);
    content.append(sectionTitle(rows.length ? `${rows.length} saved materials` : 'Saved palette'));
    if (rows.length) {
      const list = el('div', 'material-list'); for (const [name, value] of rows) { const item = el('div', 'material-row'); item.append(el('span', '', readable(name)), el('strong', '', count(value))); list.append(item); } content.append(list);
      content.append(el('p', 'detail-note', 'Counts describe the saved file, not the current live simulation.'));
    } else if (Array.isArray(row.palette_blocks) && row.palette_blocks.length) {
      content.append(el('p', 'detail-note', 'Palette presence is recorded; per-material block counts are not available.'), listNotes(row.palette_blocks.map(readable)));
    } else content.append(el('p', 'detail-note', 'Material information has not been recorded for this format.'));
  } else if (state.property === 'provenance') {
    const rows = state.detail?.entries || state.byId.get(state.selected)?.rows || [row];
    content.append(sectionTitle(`${rows.length} source ${rows.length === 1 ? 'entry' : 'entries'}`), el('p', 'detail-note', 'Identical file contents share this view. Original paths and attribution remain separate.'));
    for (const entry of rows) {
      const item = el('div', 'source-card'); item.append(el('h3', '', sourceName(entry.source)), el('code', '', entry.path));
      if (entry.designer || entry.author || entry.static?.saved_author) item.append(el('p', 'detail-note', `Attribution / saved author: ${textValue(entry.designer || entry.author || entry.static?.saved_author)}`));
      if (entry.source_commit) item.append(el('code', '', `Pinned commit ${entry.source_commit}`));
      if (isPlotPart(entry)) {
        item.append(el('p', 'detail-note', 'Derived locally from the user-provided World Record Plot. Source coordinates and saved state are retained; candidate classification is structural.'));
        if (entry.isolation) {
          item.append(el('h3', 'isolation-label', 'Isolation settings'));
          item.append(listNotes(Object.entries(entry.isolation).map(([key, value]) => `${readable(key)}: ${textValue(value)}`)));
        }
      }
      const url = safeExternal(entry.source_url || entry.url); if (url) { const link = el('a', '', 'Visit original source ↗'); link.href = url; link.target = '_blank'; link.rel = 'noopener noreferrer'; item.append(link); } else if (entry.source_url?.startsWith('file:')) item.append(el('p', 'detail-note', 'User-provided local original; no public redistribution.'));
      content.append(item);
    }
    content.append(sectionTitle('Content identity'), el('code', 'hash-label', state.selected));
  } else {
    const notes = [];
    for (const field of ['notes', 'warnings', 'review_tags', 'test_readiness_gates', 'source_claims', 'proposed_checks', 'proposed_controls']) {
      const value = row[field]; if (Array.isArray(value)) notes.push(...value); else if (value) notes.push(value);
    }
    content.append(sectionTitle('Controls & curator notes'));
    for (const field of ['finite_controls', 'control_protocol', 'test_protocol', 'observable_invariants', 'candidate_selection_method']) if (row[field]) notes.push(row[field]);
    content.append(notes.length ? listNotes(notes) : el('p', 'detail-note', 'No control protocol is recorded for this file. Static metadata alone does not establish how to operate it.'));
    if (isPlotPart(row)) {
      content.append(sectionTitle('Recorded check state'), facts([
        ['Functionality', readable(row.functional_status || 'unverified')],
        ['Simulation', readable(row.simulation_status || 'not_run')],
        ['Vanilla comparison', readable(row.vanilla_status || 'not_run')],
      ]));
      if (row.signs?.length) content.append(sectionTitle('Saved sign text'), listNotes(row.signs.map(sign => `${textValue(sign.position)} · ${textValue(sign.text)}`)));
    }
    const preview = row.thumbnail || state.byId.get(state.selected)?.row?.thumbnail;
    if (preview?.error || preview?.warnings?.length) content.append(sectionTitle('Thumbnail notes'), listNotes([preview.error, ...(preview.warnings || [])]));
    if (row.label_basis || row.evidence_basis) { content.append(sectionTitle('Label evidence')); content.append(el('p', 'detail-note', textValue(row.evidence_basis || ''))); if (row.label_basis) content.append(listNotes(Object.entries(row.label_basis).map(([key, value]) => `${key}: ${textValue(value)}`))); }
  }
}
function viewerMessage(container, titleText, description, action) {
  const node = el('div', 'viewer-message'); node.append(el('span', 'nav-icon', '⊞'), el('h3', '', titleText), el('p', '', description)); if (action) node.append(action); container.replaceChildren(node);
}
function setViewer(view) {
  state.viewer = view;
  $$('.viewer-tabs button').forEach(node => { const active = node.dataset.view === view; node.classList.toggle('active', active); node.setAttribute('aria-selected', String(active)); node.tabIndex = active ? 0 : -1; });
  for (const id of ['inspect', 'replay', 'live']) $(`#pane-${id}`).hidden = view !== id;
  $('#viewer-caption').textContent = view === 'inspect' ? 'Saved-file preview · no simulation ticks applied' : view === 'replay' ? 'Recorded capture · replay does not run the engine' : 'Live engine · properties arrive from Sim Lab';
  $('#viewer-kind').textContent = view === 'inspect' ? 'STATIC INSPECTION' : view === 'replay' ? 'RECORDED EVIDENCE' : state.liveFrame ? 'LIVE SESSION' : 'NOT STARTED';
  if (view === 'replay' && !state.replayCleanup && !state.captureAbort) loadCapture();
}
function configureCaptures() {
  const captures = state.detail?.captures || []; $('#capture-count').textContent = captures.length ? String(captures.length) : '';
  const select = $('#capture-select'); select.replaceChildren();
  for (const capture of captures) { const option = el('option', '', `${capture.label || capture.id}${capture.ticks != null ? ` · ${count(capture.ticks)} ticks` : ''}`); option.value = capture.id; select.append(option); }
  $('.capture-controls').hidden = !captures.length;
  if (!captures.length) viewerMessage($('#replay-stage'), 'No capture attached', 'This file has no retained capture available here. That is different from a passing or failing test.');
}
async function loadCapture() {
  clearReplay(); const id = $('#capture-select').value; if (!id) { configureCaptures(); return; }
  const selected = state.selected; const controller = new AbortController(); state.captureAbort = controller;
  viewerMessage($('#replay-stage'), 'Opening recorded evidence', 'Loading the saved capture and its timeline…');
  try {
    const [capture, module] = await Promise.all([getJSON(`/api/capture/${encodeURIComponent(id)}`, controller.signal), import('./replay.js')]);
    if (state.selected !== selected || controller.signal.aborted) return;
    $('#replay-stage').replaceChildren(); state.replayCleanup = await module.mountReplay($('#replay-stage'), capture);
  } catch (error) { if (error.name !== 'AbortError') viewerMessage($('#replay-stage'), 'Capture unavailable', `The recorded viewer could not open this capture. ${error.message}`); }
  finally { if (state.captureAbort === controller) state.captureAbort = null; }
}
function configureLive() {
  const available = Boolean(state.detail?.capabilities?.live_sim); $('#launch-live').disabled = !available;
  $('#launch-live').textContent = 'Open live simulation ↗';
  const unavailableReason = state.detail?.capabilities?.reason || 'The server has not advertised a usable live engine for this file.';
  const row = primaryDetail(); const large = isPlot(row) || (blocks(row) || 0) > 200000 || dimensions(row)?.reduce((a,b) => a * b, 1) > 2000000;
  $('#launch-note').className = `launch-note${large ? ' caution' : ''}`;
  $('#launch-note').textContent = !available ? `${unavailableReason} Static properties and attached captures remain available.` : large ? 'Large world file. Opening it can use substantial memory and take time. Candidate bounds are hypotheses; this launch opens the original file. Nothing starts automatically.' : 'Opens this original file in Sim Lab on demand. Import support and observed state are reported by the live engine; an open session is not a parity result.';
  viewerMessage($('#live-stage'), available ? 'Bring the mechanism to life.' : 'This file cannot launch here', available ? 'Launch explicitly to load this file into Sim Lab. The tick, entity and throughput fields below will reflect the running engine.' : unavailableReason, available ? button('Open live simulation ↗', 'button primary', launchLive) : null);
}
function launchLive() {
  if (!state.detail?.capabilities?.live_sim || !state.selected) return;
  if (!state.liveFrame) {
    const frame = document.createElement('iframe'); frame.title = `Live simulation of ${title(primaryDetail())}`; frame.src = `/sim-lab/?archive=${encodeURIComponent(state.selected)}`;
    frame.allow = 'fullscreen'; $('#live-stage').replaceChildren(frame); state.liveFrame = frame;
    $('#live-status').textContent = 'Loading'; $('#launch-live').textContent = 'Return to live simulation ↗';
  } setViewer('live');
}
function updateLive(event) {
  if (!state.liveFrame || event.origin !== location.origin || event.source !== state.liveFrame.contentWindow || event.data?.type !== 'nucleation-atlas-state') return;
  const current = event.data.state; if (!current || typeof current !== 'object' || (current.archiveSha && current.archiveSha !== state.selected)) return;
  $('#live-tick').textContent = current.currentTick ?? current.tick ?? '—';
  const tps = current.actualTps; $('#live-tps').textContent = typeof tps === 'number' && Number.isFinite(tps) ? nf.format(Math.round(tps)) : '—';
  const entityCount = Array.isArray(current.entities) ? current.entities.length : current.entities;
  $('#live-entities').textContent = typeof entityCount === 'number' ? count(entityCount) : '—';
  const statusKind = current.statusKind || current.status || '';
  $('#live-status').textContent = /error|unsupported/i.test(statusKind) ? statusKind : typeof current.running === 'boolean' ? (current.running ? 'Running' : current.quiescent ? 'Quiescent' : 'Paused') : statusKind || 'Not reported';
  const details = [current.statusMessage, typeof current.quiescent === 'boolean' ? `Quiescent: ${current.quiescent ? 'yes' : 'no'}` : '', typeof current.recording === 'boolean' ? `Recording: ${current.recording ? 'on' : 'off'}` : '', current.requestedTps != null ? `Requested TPS: ${textValue(current.requestedTps)}` : '', current.uncapped ? 'Uncapped' : ''];
  $('#engine-extra').textContent = details.filter(Boolean).join(' · ');
}
function closeDetails() { if ($('#detail').open) $('#detail').close(); }
$('#detail').addEventListener('close', () => {
  state.entryAbort?.abort(); clearReplay(); clearLive(); state.selected = null; state.detail = null;
  document.body.style.overflow = ''; if (location.hash.startsWith('#entry=')) history.replaceState(null, '', collectionURL());
  if (state.opener?.isConnected) state.opener.focus(); else $('#search').focus(); state.opener = null;
});
$('#detail').addEventListener('click', event => { const r = $('#detail').getBoundingClientRect(); if (event.target === $('#detail') && (event.clientX < r.left || event.clientX > r.right || event.clientY < r.top || event.clientY > r.bottom)) closeDetails(); });
$('#close-detail').addEventListener('click', closeDetails);
$('#launch-live').addEventListener('click', launchLive);
$('#capture-select').addEventListener('change', loadCapture);
$$('.viewer-tabs button').forEach(node => node.addEventListener('click', () => setViewer(node.dataset.view)));
$('#property-content').setAttribute('role', 'tabpanel');
$$('.property-tabs button').forEach(node => {
  node.id = `property-tab-${node.dataset.property}`; node.setAttribute('aria-controls', 'property-content');
  node.addEventListener('click', () => { state.property = node.dataset.property; renderProperties(); });
});
$$('[role=tablist]').forEach(list => list.addEventListener('keydown', event => {
  const tabs = $$('[role=tab]', list); const index = tabs.indexOf(document.activeElement);
  if (index < 0 || !['ArrowLeft','ArrowRight','Home','End'].includes(event.key)) return;
  event.preventDefault(); const next = event.key === 'Home' ? 0 : event.key === 'End' ? tabs.length - 1 : (index + (event.key === 'ArrowRight' ? 1 : -1) + tabs.length) % tabs.length;
  tabs[next].focus(); tabs[next].click();
}));
$('#search').addEventListener('input', event => { state.query = event.target.value; state.limit = PAGE_SIZE; renderResults(); });
$('#validation-filter').addEventListener('change', event => { state.filter = event.target.value; state.limit = PAGE_SIZE; renderResults(); });
$('#sort').addEventListener('change', event => { state.sort = event.target.value; state.limit = PAGE_SIZE; renderResults(); });
$('#load-more').addEventListener('click', () => { state.limit += PAGE_SIZE; renderResults(); });
for (const view of ['grid', 'table']) $(`#${view}-view`).addEventListener('click', () => { state.view = view; for (const v of ['grid', 'table']) { $(`#${v}-view`).classList.toggle('active', v === view); $(`#${v}-view`).setAttribute('aria-pressed', String(v === view)); } renderResults(); });
document.addEventListener('keydown', event => { if (event.key === '/' && !$('#detail').open && !['INPUT','TEXTAREA','SELECT'].includes(document.activeElement.tagName)) { event.preventDefault(); $('#search').focus(); } });
window.addEventListener('hashchange', handleHash); window.addEventListener('message', updateLive);

async function boot() {
  try {
    const data = await getJSON('/api/library'); state.summary = data.summary || {}; buildGroups(data.entries || []);
    renderSummary(); renderNav(); renderFeatured(); renderResults(); handleHash();
  } catch (error) {
    $('#results').setAttribute('aria-busy', 'false'); const failed = el('div', 'empty-state');
    failed.append(el('h3', '', 'The archive is not available yet.'), el('p', '', `The local API could not be reached. ${error.message}`), button('Try again', 'button secondary', boot)); $('#results').replaceChildren(failed);
    $('#summary-stats').replaceChildren(el('span', 'loading-line', 'Archive connection unavailable'));
  }
}
boot();
