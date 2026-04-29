from typing import List, Dict, Any, Optional

class Patch:
    def __init__(self):
        self.ops: List[Dict[str, Any]] = []
        self.join_rooms: List[str] = []
        self.leave_rooms: List[str] = []
        self.broadcast_instr: Optional[Dict[str, Any]] = None

    @classmethod
    def create(cls) -> 'Patch':
        return cls()

    def set_text(self, nid: str, text: str) -> 'Patch':
        self.ops.append({'op': 'set_text', 'nid': nid, 'val': text})
        return self

    def broadcast(self, b: bool = True) -> 'Patch':
        if self.ops:
            self.ops[-1]['broadcast'] = b
        return self

    def add_class(self, nid: str, class_name: str) -> 'Patch':
        self.ops.append({'op': 'add_class', 'nid': nid, 'val': class_name})
        return self

    def remove_class(self, nid: str, class_name: str) -> 'Patch':
        self.ops.append({'op': 'del_class', 'nid': nid, 'val': class_name})
        return self

    def set_style(self, nid: str, prop: str, val: str) -> 'Patch':
        self.ops.append({'op': 'set_style', 'nid': nid, 'prop': prop, 'val': val})
        return self

    def set_attr(self, nid: str, key: str, val: str) -> 'Patch':
        self.ops.append({'op': 'set_attr', 'nid': nid, 'key': key, 'val': val})
        return self

    def del_attr(self, nid: str, key: str) -> 'Patch':
        self.ops.append({'op': 'del_attr', 'nid': nid, 'key': key})
        return self

    def replace_inner(self, nid: str, html: str) -> 'Patch':
        self.ops.append({'op': 'replace_inner', 'nid': nid, 'val': html})
        return self

    def append_html(self, nid: str, html: str) -> 'Patch':
        self.ops.append({'op': 'append_html', 'nid': nid, 'val': html})
        return self

    def remove(self, nid: str) -> 'Patch':
        self.ops.append({'op': 'remove', 'nid': nid})
        return self

    def focus(self, nid: str) -> 'Patch':
        self.ops.append({'op': 'focus', 'nid': nid})
        return self

    def scroll_to(self, nid: str) -> 'Patch':
        self.ops.append({'op': 'scroll_to', 'nid': nid})
        return self

    def join_room(self, room_id: str) -> 'Patch':
        self.join_rooms.append(room_id)
        return self

    def leave_room(self, room_id: str) -> 'Patch':
        self.leave_rooms.append(room_id)
        return self

    def broadcast_to_all(self, ops: List[Dict[str, Any]]) -> 'Patch':
        self.broadcast_instr = {'scope': 'all', 'patch': ops}
        return self

    def broadcast_to_others(self, ops: List[Dict[str, Any]]) -> 'Patch':
        self.broadcast_instr = {'scope': 'others', 'patch': ops}
        return self

    def broadcast_in_room(self, room_id: str, ops: List[Dict[str, Any]]) -> 'Patch':
        self.broadcast_instr = {'scope': 'room', 'room_id': room_id, 'patch': ops}
        return self

    def broadcast_to_session(self, session_id: str, ops: List[Dict[str, Any]]) -> 'Patch':
        self.broadcast_instr = {'scope': 'direct', 'target_sid': session_id, 'patch': ops}
        return self

    def to_dict(self) -> Dict[str, Any]:
        response = {'patch': self.ops}
        if self.join_rooms:
            response['join_room'] = self.join_rooms
        if self.leave_rooms:
            response['leave_room'] = self.leave_rooms
        if self.broadcast_instr:
            response['broadcast'] = self.broadcast_instr
        return response
