


window._nhtmlState = {"site_name": "NCMS", "posts": [], "categories": [], "menu_links": [], "forms": [], "submissions": [], "current_view": "articles", "editing_post": {"id": null, "title": "", "content": "", "status": "published", "type": "post", "category_id": null, "allow_comments": 1, "form_id": null}, "editing_category": {"id": null, "name": "", "slug": "", "parent_id": null}, "editing_menu": {"id": null, "label": "", "url": "", "position": 0, "parent_id": null}};
window._nhtmlPersistent = [];
window._nhtmlWatchers = {};
window._nhtmlBindings = [];


// ── Nhtml Runtime ──────────────────────────────────────────────────────────
for (const p of (window._nhtmlPersistent || [])) {
    const saved = localStorage.getItem('nhtml_' + p);
    if (saved !== null) {
        try { window._nhtmlState[p] = JSON.parse(saved); } 
        catch(e) { window._nhtmlState[p] = saved; }
    }
}

window._nhtmlUpdateDOM = function() {
    document.querySelectorAll('[data-nhtml-text]').forEach(el => {
        try { el.textContent = window._nhtmlInterpolate(el.getAttribute('data-nhtml-text')); }
        catch(e) { console.warn('[Nhtml] text error:', e.message); }
    });
    document.querySelectorAll('[data-nhtml-html]').forEach(el => {
        try { el.innerHTML = window._nhtmlInterpolate(el.getAttribute('data-nhtml-html')); }
        catch(e) { console.warn('[Nhtml] html error:', e.message); }
    });
    document.querySelectorAll('[data-nhtml-attrs]').forEach(el => {
        const attrsJson = el.getAttribute('data-nhtml-attrs');
        let attrs;
        try { attrs = JSON.parse(attrsJson.replace(/&quot;/g, '"')); } catch(e) { return; }
        for (const [attr, template] of Object.entries(attrs)) {
            try {
                const value = window._nhtmlInterpolate(template);
                if (attr === 'color') el.style.color = value;
                else if (attr === 'background') el.style.background = value;
                else if (attr === 'disabled') el.disabled = (value === 'true' || value === true);
                else if (attr === 'visible') el.style.display = (value === 'false' || value === false) ? 'none' : '';
                else if (attr === 'value' && (el.tagName === 'INPUT' || el.tagName === 'SELECT' || el.tagName === 'TEXTAREA')) el.value = value;
                else el.setAttribute(attr, value);
            } catch(e) { console.warn('[Nhtml] attr error on', attr, ':', e.message); }
        }
    });
    document.querySelectorAll('[data-nhtml-if]').forEach(el => {
        try { el.style.display = window._nhtmlEval(el.getAttribute('data-nhtml-if')) ? '' : 'none'; }
        catch(e) { console.warn('[Nhtml] if error:', e.message); }
    });
    try {
        const titleTemplate = document.querySelector('title')?.getAttribute('data-nhtml-text');
        if (titleTemplate) document.title = window._nhtmlInterpolate(titleTemplate);
    } catch(e) {}

    for (const [varName, callbacks] of Object.entries(window._nhtmlWatchers || {})) {
        callbacks.forEach(cb => { try { cb(window._nhtmlState[varName]); } catch(e) {} });
    }
};

window._nhtmlInterpolate = function(template) {
    return template.replace(/\{(.*?)\}/g, (match, expr) => {
        const trimmed = expr.trim();
        if (/^['"]?\w+['"]?\s*:/.test(trimmed) && !trimmed.includes('?')) return match;
        return window._nhtmlEval(trimmed);
    });
};

window._nhtmlEval = function(expr) {
    try {
        const fn = new Function(...Object.keys(window._nhtmlState), `return ${expr}`);
        return fn(...Object.values(window._nhtmlState));
    } catch(e) {
        console.warn(`[Nhtml] Erreur d'évaluation pour l'expression '` + expr + `':`, e.message);
        return '';
    }
};

window.nhtml = new Proxy(window._nhtmlState, {
    set(target, prop, value) {
        target[prop] = value;
        if ((window._nhtmlPersistent || []).includes(prop)) {
            localStorage.setItem('nhtml_' + prop, JSON.stringify(value));
        }
        window._nhtmlUpdateDOM();
        document.dispatchEvent(new CustomEvent('nhtml:update', { detail: { prop, value } }));
        return true;
    },
    get(target, prop) {
        return target[prop];
    }
});

window._nhtmlWatch = function(varName, callback) {
    if (!window._nhtmlWatchers[varName]) window._nhtmlWatchers[varName] = [];
    window._nhtmlWatchers[varName].push(callback);
};

document.addEventListener('DOMContentLoaded', window._nhtmlUpdateDOM);

Object.defineProperty(window, 'site_name', {get() { return nhtml['site_name']; },set(v) { nhtml['site_name'] = v; }});
Object.defineProperty(window, 'posts', {get() { return nhtml['posts']; },set(v) { nhtml['posts'] = v; }});
Object.defineProperty(window, 'categories', {get() { return nhtml['categories']; },set(v) { nhtml['categories'] = v; }});
Object.defineProperty(window, 'menu_links', {get() { return nhtml['menu_links']; },set(v) { nhtml['menu_links'] = v; }});
Object.defineProperty(window, 'forms', {get() { return nhtml['forms']; },set(v) { nhtml['forms'] = v; }});
Object.defineProperty(window, 'submissions', {get() { return nhtml['submissions']; },set(v) { nhtml['submissions'] = v; }});
Object.defineProperty(window, 'current_view', {get() { return nhtml['current_view']; },set(v) { nhtml['current_view'] = v; }});
Object.defineProperty(window, 'editing_post', {get() { return nhtml['editing_post']; },set(v) { nhtml['editing_post'] = v; }});
Object.defineProperty(window, 'editing_category', {get() { return nhtml['editing_category']; },set(v) { nhtml['editing_category'] = v; }});
Object.defineProperty(window, 'editing_menu', {get() { return nhtml['editing_menu']; },set(v) { nhtml['editing_menu'] = v; }});


document.addEventListener('DOMContentLoaded', function() {
    function _nhtmlUpdateGroup_nhif_1() {
        const group = document.querySelectorAll('[data-nhtml-group="nhif_1"]');
        let matched = false;
        group.forEach(el => {
            if (el.hasAttribute('data-nhtml-if')) {
                const result = _nhtmlEval(el.getAttribute('data-nhtml-if'));
                el.style.display = result ? '' : 'none';
                matched = result;
            } else if (el.hasAttribute('data-nhtml-elseif')) {
                if (matched) { el.style.display = 'none'; }
                else {
                    const result = _nhtmlEval(el.getAttribute('data-nhtml-elseif'));
                    el.style.display = result ? '' : 'none';
                    matched = result;
                }
            } else if (el.hasAttribute('data-nhtml-else')) {
                el.style.display = matched ? 'none' : '';
            }
        });
    }
    _nhtmlUpdateGroup_nhif_1();
    // Re-évaluer à chaque mise à jour du state
    const _orig_nhif_1 = _nhtmlUpdateDOM;
    const _prev_nhif_1 = window._nhtmlUpdateDOM || _nhtmlUpdateDOM;
    const _saved_nhif_1 = _nhtmlUpdateDOM;
    document.addEventListener('nhtml:update', _nhtmlUpdateGroup_nhif_1);
});


document.addEventListener('DOMContentLoaded', function() {
    function _nhtmlUpdateGroup_nhif_2() {
        const group = document.querySelectorAll('[data-nhtml-group="nhif_2"]');
        let matched = false;
        group.forEach(el => {
            if (el.hasAttribute('data-nhtml-if')) {
                const result = _nhtmlEval(el.getAttribute('data-nhtml-if'));
                el.style.display = result ? '' : 'none';
                matched = result;
            } else if (el.hasAttribute('data-nhtml-elseif')) {
                if (matched) { el.style.display = 'none'; }
                else {
                    const result = _nhtmlEval(el.getAttribute('data-nhtml-elseif'));
                    el.style.display = result ? '' : 'none';
                    matched = result;
                }
            } else if (el.hasAttribute('data-nhtml-else')) {
                el.style.display = matched ? 'none' : '';
            }
        });
    }
    _nhtmlUpdateGroup_nhif_2();
    // Re-évaluer à chaque mise à jour du state
    const _orig_nhif_2 = _nhtmlUpdateDOM;
    const _prev_nhif_2 = window._nhtmlUpdateDOM || _nhtmlUpdateDOM;
    const _saved_nhif_2 = _nhtmlUpdateDOM;
    document.addEventListener('nhtml:update', _nhtmlUpdateGroup_nhif_2);
});


document.addEventListener('DOMContentLoaded', function() {
    function _nhtmlUpdateGroup_nhif_3() {
        const group = document.querySelectorAll('[data-nhtml-group="nhif_3"]');
        let matched = false;
        group.forEach(el => {
            if (el.hasAttribute('data-nhtml-if')) {
                const result = _nhtmlEval(el.getAttribute('data-nhtml-if'));
                el.style.display = result ? '' : 'none';
                matched = result;
            } else if (el.hasAttribute('data-nhtml-elseif')) {
                if (matched) { el.style.display = 'none'; }
                else {
                    const result = _nhtmlEval(el.getAttribute('data-nhtml-elseif'));
                    el.style.display = result ? '' : 'none';
                    matched = result;
                }
            } else if (el.hasAttribute('data-nhtml-else')) {
                el.style.display = matched ? 'none' : '';
            }
        });
    }
    _nhtmlUpdateGroup_nhif_3();
    // Re-évaluer à chaque mise à jour du state
    const _orig_nhif_3 = _nhtmlUpdateDOM;
    const _prev_nhif_3 = window._nhtmlUpdateDOM || _nhtmlUpdateDOM;
    const _saved_nhif_3 = _nhtmlUpdateDOM;
    document.addEventListener('nhtml:update', _nhtmlUpdateGroup_nhif_3);
});


document.addEventListener('DOMContentLoaded', function() {
    function _nhtmlUpdateGroup_nhif_4() {
        const group = document.querySelectorAll('[data-nhtml-group="nhif_4"]');
        let matched = false;
        group.forEach(el => {
            if (el.hasAttribute('data-nhtml-if')) {
                const result = _nhtmlEval(el.getAttribute('data-nhtml-if'));
                el.style.display = result ? '' : 'none';
                matched = result;
            } else if (el.hasAttribute('data-nhtml-elseif')) {
                if (matched) { el.style.display = 'none'; }
                else {
                    const result = _nhtmlEval(el.getAttribute('data-nhtml-elseif'));
                    el.style.display = result ? '' : 'none';
                    matched = result;
                }
            } else if (el.hasAttribute('data-nhtml-else')) {
                el.style.display = matched ? 'none' : '';
            }
        });
    }
    _nhtmlUpdateGroup_nhif_4();
    // Re-évaluer à chaque mise à jour du state
    const _orig_nhif_4 = _nhtmlUpdateDOM;
    const _prev_nhif_4 = window._nhtmlUpdateDOM || _nhtmlUpdateDOM;
    const _saved_nhif_4 = _nhtmlUpdateDOM;
    document.addEventListener('nhtml:update', _nhtmlUpdateGroup_nhif_4);
});


document.addEventListener('DOMContentLoaded', function() {
    function _nhtmlUpdateGroup_nhif_5() {
        const group = document.querySelectorAll('[data-nhtml-group="nhif_5"]');
        let matched = false;
        group.forEach(el => {
            if (el.hasAttribute('data-nhtml-if')) {
                const result = _nhtmlEval(el.getAttribute('data-nhtml-if'));
                el.style.display = result ? '' : 'none';
                matched = result;
            } else if (el.hasAttribute('data-nhtml-elseif')) {
                if (matched) { el.style.display = 'none'; }
                else {
                    const result = _nhtmlEval(el.getAttribute('data-nhtml-elseif'));
                    el.style.display = result ? '' : 'none';
                    matched = result;
                }
            } else if (el.hasAttribute('data-nhtml-else')) {
                el.style.display = matched ? 'none' : '';
            }
        });
    }
    _nhtmlUpdateGroup_nhif_5();
    // Re-évaluer à chaque mise à jour du state
    const _orig_nhif_5 = _nhtmlUpdateDOM;
    const _prev_nhif_5 = window._nhtmlUpdateDOM || _nhtmlUpdateDOM;
    const _saved_nhif_5 = _nhtmlUpdateDOM;
    document.addEventListener('nhtml:update', _nhtmlUpdateGroup_nhif_5);
});


document.addEventListener('DOMContentLoaded', function() {
    function _nhtmlUpdateGroup_nhif_6() {
        const group = document.querySelectorAll('[data-nhtml-group="nhif_6"]');
        let matched = false;
        group.forEach(el => {
            if (el.hasAttribute('data-nhtml-if')) {
                const result = _nhtmlEval(el.getAttribute('data-nhtml-if'));
                el.style.display = result ? '' : 'none';
                matched = result;
            } else if (el.hasAttribute('data-nhtml-elseif')) {
                if (matched) { el.style.display = 'none'; }
                else {
                    const result = _nhtmlEval(el.getAttribute('data-nhtml-elseif'));
                    el.style.display = result ? '' : 'none';
                    matched = result;
                }
            } else if (el.hasAttribute('data-nhtml-else')) {
                el.style.display = matched ? 'none' : '';
            }
        });
    }
    _nhtmlUpdateGroup_nhif_6();
    // Re-évaluer à chaque mise à jour du state
    const _orig_nhif_6 = _nhtmlUpdateDOM;
    const _prev_nhif_6 = window._nhtmlUpdateDOM || _nhtmlUpdateDOM;
    const _saved_nhif_6 = _nhtmlUpdateDOM;
    document.addEventListener('nhtml:update', _nhtmlUpdateGroup_nhif_6);
});


document.addEventListener('DOMContentLoaded', function() {
    function _nhtmlUpdateGroup_nhif_7() {
        const group = document.querySelectorAll('[data-nhtml-group="nhif_7"]');
        let matched = false;
        group.forEach(el => {
            if (el.hasAttribute('data-nhtml-if')) {
                const result = _nhtmlEval(el.getAttribute('data-nhtml-if'));
                el.style.display = result ? '' : 'none';
                matched = result;
            } else if (el.hasAttribute('data-nhtml-elseif')) {
                if (matched) { el.style.display = 'none'; }
                else {
                    const result = _nhtmlEval(el.getAttribute('data-nhtml-elseif'));
                    el.style.display = result ? '' : 'none';
                    matched = result;
                }
            } else if (el.hasAttribute('data-nhtml-else')) {
                el.style.display = matched ? 'none' : '';
            }
        });
    }
    _nhtmlUpdateGroup_nhif_7();
    // Re-évaluer à chaque mise à jour du state
    const _orig_nhif_7 = _nhtmlUpdateDOM;
    const _prev_nhif_7 = window._nhtmlUpdateDOM || _nhtmlUpdateDOM;
    const _saved_nhif_7 = _nhtmlUpdateDOM;
    document.addEventListener('nhtml:update', _nhtmlUpdateGroup_nhif_7);
});


document.addEventListener('DOMContentLoaded', function() {
    function _nhtmlRender_nheach_8() {
        const container = document.getElementById('nheach_8');
        if (!container) return;
        
        // Support des expressions complexes dans in="..."
        let items = [];
        try {
            items = _nhtmlEval('posts.filter(p => p.type === \'post\')');
            if (!Array.isArray(items)) items = [];
        } catch(e) {
            console.warn("[Nhtml] Erreur boucle each:", e);
        }
        
        

        container.innerHTML = '';
        items.forEach((_item, _i) => {
            
            const _nhtml_loop_item_nheach_8 = _item;
            const _tpl = `<tr>
                        <td style="font-weight:bold;"><span data-nhtml-text="${_nhtml_loop_item_nheach_8.title}">${_nhtml_loop_item_nheach_8.title}</span></td>
                        <td><small><span data-nhtml-text="${_nhtml_loop_item_nheach_8.category_name || '-'}">${_nhtml_loop_item_nheach_8.category_name || '-'}</span></small></td>
                        <td><span  data-nhtml-attrs="{&quot;class&quot;: &quot;status ${_nhtml_loop_item_nheach_8.status}&quot;}"><span data-nhtml-text="${_nhtml_loop_item_nheach_8.status}">${_nhtml_loop_item_nheach_8.status}</span></span></td>
                        <td><i  data-nhtml-attrs="{&quot;class&quot;: &quot;fas ${_nhtml_loop_item_nheach_8.allow_comments == 1 ? 'fa-comment' : 'fa-comment-slash'}&quot;}"></i></td>
                        <td>
                            <a class="btn-icon" data-nhtml-attrs="{&quot;href&quot;: &quot;/admin/edit/${_nhtml_loop_item_nheach_8.id}&quot;}"><i class="fas fa-edit"></i></a>
                            <form action="/admin/delete" method="POST" style="display:inline;">
                                <input type="hidden" name="id" data-nhtml-attrs="{&quot;value&quot;: &quot;${_nhtml_loop_item_nheach_8.id}&quot;}"/><input type="hidden" name="type" value="post"/>
                                <button type="submit" class="danger"><i class="fas fa-trash"></i></button>
                            </form>
                        </td>
                    </tr>`;
            container.insertAdjacentHTML('beforeend', _tpl);
        });
        // Mettre à jour le marqueur empty
        const empty = document.querySelector('[data-nhtml-empty="nheach_8"]');
        if (empty) empty.style.display = (items && items.length === 0) ? '' : 'none';
        
        if (typeof _nhtmlUpdateDOM === 'function') {
            _nhtmlUpdateDOM();
        }
    }
    _nhtmlRender_nheach_8();
    document.addEventListener('nhtml:update', _nhtmlRender_nheach_8);
});


document.addEventListener('DOMContentLoaded', function() {
    function _nhtmlRender_nheach_9() {
        const container = document.getElementById('nheach_9');
        if (!container) return;
        
        // Support des expressions complexes dans in="..."
        let items = [];
        try {
            items = _nhtmlEval('posts.filter(p => p.type === \'page\')');
            if (!Array.isArray(items)) items = [];
        } catch(e) {
            console.warn("[Nhtml] Erreur boucle each:", e);
        }
        
        

        container.innerHTML = '';
        items.forEach((_item, _i) => {
            
            const _nhtml_loop_item_nheach_9 = _item;
            const _tpl = `<tr>
                        <td style="font-weight:bold;"><span data-nhtml-text="${_nhtml_loop_item_nheach_9.title}">${_nhtml_loop_item_nheach_9.title}</span></td>
                        <td><small><span data-nhtml-text="${_nhtml_loop_item_nheach_9.form_id ? '✅ Lié' : '-'}">${_nhtml_loop_item_nheach_9.form_id ? '✅ Lié' : '-'}</span></small></td>
                        <td><span  data-nhtml-attrs="{&quot;class&quot;: &quot;status ${_nhtml_loop_item_nheach_9.status}&quot;}"><span data-nhtml-text="${_nhtml_loop_item_nheach_9.status}">${_nhtml_loop_item_nheach_9.status}</span></span></td>
                        <td>
                            <a class="btn-icon" data-nhtml-attrs="{&quot;href&quot;: &quot;/admin/edit/${_nhtml_loop_item_nheach_9.id}&quot;}"><i class="fas fa-edit"></i></a>
                            <form action="/admin/delete" method="POST" style="display:inline;">
                                <input type="hidden" name="id" data-nhtml-attrs="{&quot;value&quot;: &quot;${_nhtml_loop_item_nheach_9.id}&quot;}"/><input type="hidden" name="type" value="page"/>
                                <button type="submit" class="danger"><i class="fas fa-trash"></i></button>
                            </form>
                        </td>
                    </tr>`;
            container.insertAdjacentHTML('beforeend', _tpl);
        });
        // Mettre à jour le marqueur empty
        const empty = document.querySelector('[data-nhtml-empty="nheach_9"]');
        if (empty) empty.style.display = (items && items.length === 0) ? '' : 'none';
        
        if (typeof _nhtmlUpdateDOM === 'function') {
            _nhtmlUpdateDOM();
        }
    }
    _nhtmlRender_nheach_9();
    document.addEventListener('nhtml:update', _nhtmlRender_nheach_9);
});


document.addEventListener('DOMContentLoaded', function() {
    function _nhtmlRender_nheach_10() {
        const container = document.getElementById('nheach_10');
        if (!container) return;
        
        // Support des expressions complexes dans in="..."
        let items = [];
        try {
            items = _nhtmlEval('categories');
            if (!Array.isArray(items)) items = [];
        } catch(e) {
            console.warn("[Nhtml] Erreur boucle each:", e);
        }
        
        

        container.innerHTML = '';
        items.forEach((_item, _i) => {
            
            const _nhtml_loop_item_nheach_10 = _item;
            const _tpl = `<option  data-nhtml-attrs="{&quot;value&quot;: &quot;${_nhtml_loop_item_nheach_10.id}&quot;}"><span data-nhtml-text="${_nhtml_loop_item_nheach_10.name}">${_nhtml_loop_item_nheach_10.name}</span></option>`;
            container.insertAdjacentHTML('beforeend', _tpl);
        });
        // Mettre à jour le marqueur empty
        const empty = document.querySelector('[data-nhtml-empty="nheach_10"]');
        if (empty) empty.style.display = (items && items.length === 0) ? '' : 'none';
        
        if (typeof _nhtmlUpdateDOM === 'function') {
            _nhtmlUpdateDOM();
        }
    }
    _nhtmlRender_nheach_10();
    document.addEventListener('nhtml:update', _nhtmlRender_nheach_10);
});


document.addEventListener('DOMContentLoaded', function() {
    function _nhtmlRender_nheach_11() {
        const container = document.getElementById('nheach_11');
        if (!container) return;
        
        // Support des expressions complexes dans in="..."
        let items = [];
        try {
            items = _nhtmlEval('categories');
            if (!Array.isArray(items)) items = [];
        } catch(e) {
            console.warn("[Nhtml] Erreur boucle each:", e);
        }
        
        

        container.innerHTML = '';
        items.forEach((_item, _i) => {
            
            const _nhtml_loop_item_nheach_11 = _item;
            const _tpl = `<tr>
                        <td><span data-nhtml-text="${_nhtml_loop_item_nheach_11.parent_id ? '↳ ' : ''}">${_nhtml_loop_item_nheach_11.parent_id ? '↳ ' : ''}</span><strong><span data-nhtml-text="${_nhtml_loop_item_nheach_11.name}">${_nhtml_loop_item_nheach_11.name}</span></strong></td>
                        <td><code><span data-nhtml-text="${_nhtml_loop_item_nheach_11.slug || '-'}">${_nhtml_loop_item_nheach_11.slug || '-'}</span></code></td>
                        <td>
                            <button type="button" class="secondary" title="Éditer" onclick="editing_category.id=${_nhtml_loop_item_nheach_11.id}; editing_category.name='${_nhtml_loop_item_nheach_11.name.replace(\`'\`, \`\\'\`).replace(\`"\`, \`\\"\`)}'; editing_category.slug='${_nhtml_loop_item_nheach_11.slug}'; editing_category.parent_id='${_nhtml_loop_item_nheach_11.parent_id || ''}'; window.scrollTo(0,0)"><i class="fas fa-edit"></i></button>
                            <form action="/admin/category/delete" method="POST" style="display:inline;"><input type="hidden" name="id" data-nhtml-attrs="{&quot;value&quot;: &quot;${_nhtml_loop_item_nheach_11.id}&quot;}"/><button class="danger"><i class="fas fa-trash"></i></button></form>
                        </td>
                    </tr>`;
            container.insertAdjacentHTML('beforeend', _tpl);
        });
        // Mettre à jour le marqueur empty
        const empty = document.querySelector('[data-nhtml-empty="nheach_11"]');
        if (empty) empty.style.display = (items && items.length === 0) ? '' : 'none';
        
        if (typeof _nhtmlUpdateDOM === 'function') {
            _nhtmlUpdateDOM();
        }
    }
    _nhtmlRender_nheach_11();
    document.addEventListener('nhtml:update', _nhtmlRender_nheach_11);
});


document.addEventListener('DOMContentLoaded', function() {
    function _nhtmlRender_nheach_12() {
        const container = document.getElementById('nheach_12');
        if (!container) return;
        
        // Support des expressions complexes dans in="..."
        let items = [];
        try {
            items = _nhtmlEval('categories');
            if (!Array.isArray(items)) items = [];
        } catch(e) {
            console.warn("[Nhtml] Erreur boucle each:", e);
        }
        
        

        container.innerHTML = '';
        items.forEach((_item, _i) => {
            
            const _nhtml_loop_item_nheach_12 = _item;
            const _tpl = `<option  data-nhtml-attrs="{&quot;value&quot;: &quot;${_nhtml_loop_item_nheach_12.id}&quot;}"><span data-nhtml-text="${_nhtml_loop_item_nheach_12.name}">${_nhtml_loop_item_nheach_12.name}</span></option>`;
            container.insertAdjacentHTML('beforeend', _tpl);
        });
        // Mettre à jour le marqueur empty
        const empty = document.querySelector('[data-nhtml-empty="nheach_12"]');
        if (empty) empty.style.display = (items && items.length === 0) ? '' : 'none';
        
        if (typeof _nhtmlUpdateDOM === 'function') {
            _nhtmlUpdateDOM();
        }
    }
    _nhtmlRender_nheach_12();
    document.addEventListener('nhtml:update', _nhtmlRender_nheach_12);
});


document.addEventListener('DOMContentLoaded', function() {
    function _nhtmlRender_nheach_13() {
        const container = document.getElementById('nheach_13');
        if (!container) return;
        
        // Support des expressions complexes dans in="..."
        let items = [];
        try {
            items = _nhtmlEval('forms');
            if (!Array.isArray(items)) items = [];
        } catch(e) {
            console.warn("[Nhtml] Erreur boucle each:", e);
        }
        
        

        container.innerHTML = '';
        items.forEach((_item, _i) => {
            
            const _nhtml_loop_item_nheach_13 = _item;
            const _tpl = `<option  data-nhtml-attrs="{&quot;value&quot;: &quot;${_nhtml_loop_item_nheach_13.id}&quot;}"><span data-nhtml-text="${_nhtml_loop_item_nheach_13.name}">${_nhtml_loop_item_nheach_13.name}</span></option>`;
            container.insertAdjacentHTML('beforeend', _tpl);
        });
        // Mettre à jour le marqueur empty
        const empty = document.querySelector('[data-nhtml-empty="nheach_13"]');
        if (empty) empty.style.display = (items && items.length === 0) ? '' : 'none';
        
        if (typeof _nhtmlUpdateDOM === 'function') {
            _nhtmlUpdateDOM();
        }
    }
    _nhtmlRender_nheach_13();
    document.addEventListener('nhtml:update', _nhtmlRender_nheach_13);
});


document.addEventListener('DOMContentLoaded', function() {
    function _nhtmlRender_nheach_14() {
        const container = document.getElementById('nheach_14');
        if (!container) return;
        
        // Support des expressions complexes dans in="..."
        let items = [];
        try {
            items = _nhtmlEval('posts.filter(p => p.type === \'page\')');
            if (!Array.isArray(items)) items = [];
        } catch(e) {
            console.warn("[Nhtml] Erreur boucle each:", e);
        }
        
        

        container.innerHTML = '';
        items.forEach((_item, _i) => {
            
            const _nhtml_loop_item_nheach_14 = _item;
            const _tpl = `<li style="display:flex; justify-content:space-between; align-items:center; padding:8px 0; border-bottom:1px solid #eee;">
                                <strong><span data-nhtml-text="${_nhtml_loop_item_nheach_14.title}">${_nhtml_loop_item_nheach_14.title}</span></strong>
                                <form action="/admin/menu/save" method="POST" style="margin:0;">
                                    <input type="hidden" name="label" data-nhtml-attrs="{&quot;value&quot;: &quot;${_nhtml_loop_item_nheach_14.title}&quot;}"/>
                                    <input type="hidden" name="url" data-nhtml-attrs="{&quot;value&quot;: &quot;/page/${_nhtml_loop_item_nheach_14.id}&quot;}"/>
                                    <button type="submit" class="secondary" style="padding:4px 8px; font-size:0.85rem;"><i class="fas fa-plus"></i></button>
                                </form>
                            </li>`;
            container.insertAdjacentHTML('beforeend', _tpl);
        });
        // Mettre à jour le marqueur empty
        const empty = document.querySelector('[data-nhtml-empty="nheach_14"]');
        if (empty) empty.style.display = (items && items.length === 0) ? '' : 'none';
        
        if (typeof _nhtmlUpdateDOM === 'function') {
            _nhtmlUpdateDOM();
        }
    }
    _nhtmlRender_nheach_14();
    document.addEventListener('nhtml:update', _nhtmlRender_nheach_14);
});


document.addEventListener('DOMContentLoaded', function() {
    function _nhtmlRender_nheach_15() {
        const container = document.getElementById('nheach_15');
        if (!container) return;
        
        // Support des expressions complexes dans in="..."
        let items = [];
        try {
            items = _nhtmlEval('menu_links');
            if (!Array.isArray(items)) items = [];
        } catch(e) {
            console.warn("[Nhtml] Erreur boucle each:", e);
        }
        
        

        container.innerHTML = '';
        items.forEach((_item, _i) => {
            
            const _nhtml_loop_item_nheach_15 = _item;
            const _tpl = `<option  data-nhtml-attrs="{&quot;value&quot;: &quot;${_nhtml_loop_item_nheach_15.id}&quot;}"><span data-nhtml-text="${_nhtml_loop_item_nheach_15.label}">${_nhtml_loop_item_nheach_15.label}</span></option>`;
            container.insertAdjacentHTML('beforeend', _tpl);
        });
        // Mettre à jour le marqueur empty
        const empty = document.querySelector('[data-nhtml-empty="nheach_15"]');
        if (empty) empty.style.display = (items && items.length === 0) ? '' : 'none';
        
        if (typeof _nhtmlUpdateDOM === 'function') {
            _nhtmlUpdateDOM();
        }
    }
    _nhtmlRender_nheach_15();
    document.addEventListener('nhtml:update', _nhtmlRender_nheach_15);
});


document.addEventListener('DOMContentLoaded', function() {
    function _nhtmlRender_nheach_16() {
        const container = document.getElementById('nheach_16');
        if (!container) return;
        
        // Support des expressions complexes dans in="..."
        let items = [];
        try {
            items = _nhtmlEval('menu_links');
            if (!Array.isArray(items)) items = [];
        } catch(e) {
            console.warn("[Nhtml] Erreur boucle each:", e);
        }
        
        

        container.innerHTML = '';
        items.forEach((_item, _i) => {
            
            const _nhtml_loop_item_nheach_16 = _item;
            const _tpl = `<tr>
                        <td><span data-nhtml-text="${_nhtml_loop_item_nheach_16.parent_id ? '↳ ' : ''}">${_nhtml_loop_item_nheach_16.parent_id ? '↳ ' : ''}</span><strong><span data-nhtml-text="${_nhtml_loop_item_nheach_16.label}">${_nhtml_loop_item_nheach_16.label}</span></strong></td>
                        <td><code><span data-nhtml-text="${_nhtml_loop_item_nheach_16.url}">${_nhtml_loop_item_nheach_16.url}</span></code></td>
                        <td><span data-nhtml-text="${_nhtml_loop_item_nheach_16.position}">${_nhtml_loop_item_nheach_16.position}</span></td>
                        <td>
                            <button type="button" class="secondary" title="Éditer" onclick="editing_menu.id=${_nhtml_loop_item_nheach_16.id}; editing_menu.label='${_nhtml_loop_item_nheach_16.label.replace(\`'\`, \`\\'\`).replace(\`"\`, \`\\"\`)}'; editing_menu.url='${_nhtml_loop_item_nheach_16.url}'; editing_menu.position=${_nhtml_loop_item_nheach_16.position}; editing_menu.parent_id='${_nhtml_loop_item_nheach_16.parent_id || ''}'; window.scrollTo(0, 0);"><i class="fas fa-edit"></i></button>
                            <form action="/admin/menu/delete" method="POST" style="display:inline;"><input type="hidden" name="id" data-nhtml-attrs="{&quot;value&quot;: &quot;${_nhtml_loop_item_nheach_16.id}&quot;}"/><button class="danger"><i class="fas fa-trash"></i></button></form>
                        </td>
                    </tr>`;
            container.insertAdjacentHTML('beforeend', _tpl);
        });
        // Mettre à jour le marqueur empty
        const empty = document.querySelector('[data-nhtml-empty="nheach_16"]');
        if (empty) empty.style.display = (items && items.length === 0) ? '' : 'none';
        
        if (typeof _nhtmlUpdateDOM === 'function') {
            _nhtmlUpdateDOM();
        }
    }
    _nhtmlRender_nheach_16();
    document.addEventListener('nhtml:update', _nhtmlRender_nheach_16);
});


document.addEventListener('DOMContentLoaded', function() {
    function _nhtmlRender_nheach_17() {
        const container = document.getElementById('nheach_17');
        if (!container) return;
        
        // Support des expressions complexes dans in="..."
        let items = [];
        try {
            items = _nhtmlEval('forms');
            if (!Array.isArray(items)) items = [];
        } catch(e) {
            console.warn("[Nhtml] Erreur boucle each:", e);
        }
        
        

        container.innerHTML = '';
        items.forEach((_item, _i) => {
            
            const _nhtml_loop_item_nheach_17 = _item;
            const _tpl = `<tr>
                        <td><strong><span data-nhtml-text="${_nhtml_loop_item_nheach_17.name}">${_nhtml_loop_item_nheach_17.name}</span></strong></td>
                        <td><small><span data-nhtml-text="${_nhtml_loop_item_nheach_17.fields ? JSON.parse(_nhtml_loop_item_nheach_17.fields).length : 0}">${_nhtml_loop_item_nheach_17.fields ? JSON.parse(_nhtml_loop_item_nheach_17.fields).length : 0}</span> champs</small></td>
                        <td><form action="/admin/form/delete" method="POST" style="display:inline;"><input type="hidden" name="id" data-nhtml-attrs="{&quot;value&quot;: &quot;${_nhtml_loop_item_nheach_17.id}&quot;}"/><button class="danger"><i class="fas fa-trash"></i></button></form></td>
                    </tr>`;
            container.insertAdjacentHTML('beforeend', _tpl);
        });
        // Mettre à jour le marqueur empty
        const empty = document.querySelector('[data-nhtml-empty="nheach_17"]');
        if (empty) empty.style.display = (items && items.length === 0) ? '' : 'none';
        
        if (typeof _nhtmlUpdateDOM === 'function') {
            _nhtmlUpdateDOM();
        }
    }
    _nhtmlRender_nheach_17();
    document.addEventListener('nhtml:update', _nhtmlRender_nheach_17);
});


document.addEventListener('DOMContentLoaded', function() {
    function _nhtmlRender_nheach_18() {
        const container = document.getElementById('nheach_18');
        if (!container) return;
        
        // Support des expressions complexes dans in="..."
        let items = [];
        try {
            items = _nhtmlEval('submissions');
            if (!Array.isArray(items)) items = [];
        } catch(e) {
            console.warn("[Nhtml] Erreur boucle each:", e);
        }
        
        

        container.innerHTML = '';
        items.forEach((_item, _i) => {
            
            const _nhtml_loop_item_nheach_18 = _item;
            const _tpl = `<div class="submission-card">
                    <div class="s-header"><strong><span data-nhtml-text="${_nhtml_loop_item_nheach_18.form_name}">${_nhtml_loop_item_nheach_18.form_name}</span></strong> <small>le <span data-nhtml-text="${_nhtml_loop_item_nheach_18.created_at}">${_nhtml_loop_item_nheach_18.created_at}</span></small></div>
                    <pre><span data-nhtml-text="${JSON.stringify(JSON.parse(_nhtml_loop_item_nheach_18.data), null, 2)}">${JSON.stringify(JSON.parse(_nhtml_loop_item_nheach_18.data), null, 2)}</span></pre>
                </div>`;
            container.insertAdjacentHTML('beforeend', _tpl);
        });
        // Mettre à jour le marqueur empty
        const empty = document.querySelector('[data-nhtml-empty="nheach_18"]');
        if (empty) empty.style.display = (items && items.length === 0) ? '' : 'none';
        
        if (typeof _nhtmlUpdateDOM === 'function') {
            _nhtmlUpdateDOM();
        }
    }
    _nhtmlRender_nheach_18();
    document.addEventListener('nhtml:update', _nhtmlRender_nheach_18);
});


        let editorInstance = null;

        function initEditor(content) {
            const el = document.getElementById('editor');
            if (!el) return;
            el.innerHTML = '';
            editorInstance = pell.init({
                element: el,
                onChange: html => {
                    const ta = document.getElementById('hidden-content');
                    if (ta) ta.value = html;
                },
                defaultParagraphSeparator: 'p',
                actions: ['bold', 'italic', 'underline', 'strikethrough', 'heading1', 'heading2', 'paragraph', 'quote', 'olist', 'ulist', 'link']
            });
            editorInstance.content.innerHTML = content || '';
        }

        function syncEditor() {
            if (editorInstance) {
                const ta = document.getElementById('hidden-content');
                if (ta) ta.value = editorInstance.content.innerHTML;
            }
        }

        // Auto-init de pell et pré-remplissage des selects au chargement
        document.addEventListener('DOMContentLoaded', function() {
            const view = nhtml['current_view'];
            if (view === 'edit' || view === 'new') {
                const post = nhtml['editing_post'] || {};
                initEditor(post.content || '');

                // Pre-fill selects (Nhtml reactive ne gère pas encore le 'selected')
                const typeSelect = document.querySelector('select[name="type"]');
                if (typeSelect && post.type) typeSelect.value = post.type;

                const statusSelect = document.querySelector('select[name="status"]');
                if (statusSelect && post.status) statusSelect.value = post.status;

                const catSelect = document.querySelector('select[name="category_id"]');
                if (catSelect && post.category_id) catSelect.value = post.category_id;

                const commSelect = document.querySelector('select[name="allow_comments"]');
                if (commSelect) commSelect.value = post.allow_comments !== undefined ? post.allow_comments : 1;

                const formSelect = document.querySelector('select[name="form_id"]');
                if (formSelect && post.form_id) formSelect.value = post.form_id;
            }
        });
    
