package nhtml

type PatchOp map[string]interface{}

type BroadcastInstruction struct {
	Scope     string    `json:"scope"`
	RoomID    string    `json:"room_id,omitempty"`
	TargetSID string    `json:"target_sid,omitempty"`
	Patch     []PatchOp `json:"patch"`
}

type Patch struct {
	Ops            []PatchOp             `json:"patch"`
	JoinRooms      []string              `json:"join_room,omitempty"`
	LeaveRooms     []string              `json:"leave_room,omitempty"`
	BroadcastInstr *BroadcastInstruction `json:"broadcast,omitempty"`
}

func NewPatch() *Patch {
	return &Patch{
		Ops:        []PatchOp{},
		JoinRooms:  []string{},
		LeaveRooms: []string{},
	}
}

func (p *Patch) SetText(nid string, text string) *Patch {
	p.Ops = append(p.Ops, PatchOp{"op": "set_text", "nid": nid, "val": text})
	return p
}

func (p *Patch) Broadcast(b bool) *Patch {
	if len(p.Ops) > 0 {
		p.Ops[len(p.Ops)-1]["broadcast"] = b
	}
	return p
}

func (p *Patch) AddClass(nid string, className string) *Patch {
	p.Ops = append(p.Ops, PatchOp{"op": "add_class", "nid": nid, "val": className})
	return p
}

func (p *Patch) RemoveClass(nid string, className string) *Patch {
	p.Ops = append(p.Ops, PatchOp{"op": "del_class", "nid": nid, "val": className})
	return p
}

func (p *Patch) SetStyle(nid string, prop string, val string) *Patch {
	p.Ops = append(p.Ops, PatchOp{"op": "set_style", "nid": nid, "prop": prop, "val": val})
	return p
}

func (p *Patch) SetAttr(nid string, key string, val string) *Patch {
	p.Ops = append(p.Ops, PatchOp{"op": "set_attr", "nid": nid, "key": key, "val": val})
	return p
}

func (p *Patch) DelAttr(nid string, key string) *Patch {
	p.Ops = append(p.Ops, PatchOp{"op": "del_attr", "nid": nid, "key": key})
	return p
}

func (p *Patch) ReplaceInner(nid string, html string) *Patch {
	p.Ops = append(p.Ops, PatchOp{"op": "replace_inner", "nid": nid, "val": html})
	return p
}

func (p *Patch) AppendHTML(nid string, html string) *Patch {
	p.Ops = append(p.Ops, PatchOp{"op": "append_html", "nid": nid, "val": html})
	return p
}

func (p *Patch) Remove(nid string) *Patch {
	p.Ops = append(p.Ops, PatchOp{"op": "remove", "nid": nid})
	return p
}

func (p *Patch) Focus(nid string) *Patch {
	p.Ops = append(p.Ops, PatchOp{"op": "focus", "nid": nid})
	return p
}

func (p *Patch) ScrollTo(nid string) *Patch {
	p.Ops = append(p.Ops, PatchOp{"op": "scroll_to", "nid": nid})
	return p
}

func (p *Patch) JoinRoom(roomID string) *Patch {
	p.JoinRooms = append(p.JoinRooms, roomID)
	return p
}

func (p *Patch) LeaveRoom(roomID string) *Patch {
	p.LeaveRooms = append(p.LeaveRooms, roomID)
	return p
}

func (p *Patch) BroadcastToAll(ops []PatchOp) *Patch {
	p.BroadcastInstr = &BroadcastInstruction{Scope: "all", Patch: ops}
	return p
}

func (p *Patch) BroadcastToOthers(ops []PatchOp) *Patch {
	p.BroadcastInstr = &BroadcastInstruction{Scope: "others", Patch: ops}
	return p
}

func (p *Patch) BroadcastInRoom(roomID string, ops []PatchOp) *Patch {
	p.BroadcastInstr = &BroadcastInstruction{Scope: "room", RoomID: roomID, Patch: ops}
	return p
}

func (p *Patch) BroadcastToSession(sessionID string, ops []PatchOp) *Patch {
	p.BroadcastInstr = &BroadcastInstruction{Scope: "direct", TargetSID: sessionID, Patch: ops}
	return p
}
