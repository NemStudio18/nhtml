/**
 * NHTML SDK Node.js v0.7.1
 */

class Patch {
    constructor() {
        this.ops = [];
        this.joinRooms = [];
        this.leaveRooms = [];
        this.broadcastInstr = null;
    }

    static create() {
        return new Patch();
    }

    setText(nid, text) {
        this.ops.push({ op: 'set_text', nid, val: text });
        return this;
    }

    addClass(nid, className) {
        this.ops.push({ op: 'add_class', nid, val: className });
        return this;
    }

    removeClass(nid, className) {
        this.ops.push({ op: 'del_class', nid, val: className });
        return this;
    }

    setStyle(nid, prop, val) {
        this.ops.push({ op: 'set_style', nid, prop, val });
        return this;
    }

    setAttr(nid, key, val) {
        this.ops.push({ op: 'set_attr', nid, key, val });
        return this;
    }

    delAttr(nid, key) {
        this.ops.push({ op: 'del_attr', nid, key });
        return this;
    }

    replaceInner(nid, html) {
        this.ops.push({ op: 'replace_inner', nid, val: html });
        return this;
    }

    appendHtml(nid, html) {
        this.ops.push({ op: 'append_html', nid, val: html });
        return this;
    }

    remove(nid) {
        this.ops.push({ op: 'remove', nid });
        return this;
    }

    focus(nid) {
        this.ops.push({ op: 'focus', nid });
        return this;
    }

    scrollTo(nid) {
        this.ops.push({ op: 'scroll_to', nid });
        return this;
    }

    joinRoom(roomId) {
        this.joinRooms.push(roomId);
        return this;
    }

    leaveRoom(roomId) {
        this.leaveRooms.push(roomId);
        return this;
    }

    broadcastToAll(ops) {
        this.broadcastInstr = { scope: 'all', patch: ops };
        return this;
    }

    toDict() {
        const response = { patch: this.ops };
        if (this.joinRooms.length > 0) response.join_room = this.joinRooms;
        if (this.leaveRooms.length > 0) response.leave_room = this.leaveRooms;
        if (this.broadcastInstr) response.broadcast = this.broadcastInstr;
        return response;
    }

    // Express helper
    send(res) {
        res.json(this.toDict());
    }
}

module.exports = {
    Patch,
    parseEvent: (body) => ({
        nodeId: body['n-id'],
        handler: body['n-handler'],
        value: body['n-value'],
        type: body['n-type'],
        sessionId: body['n-session-id']
    })
};
