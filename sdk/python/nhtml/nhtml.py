import json
from typing import List, Dict, Any, Optional, Callable
from .patch import Patch

class Nhtml:
    @staticmethod
    def patch() -> Patch:
        """Creates a new patch collection."""
        return Patch.create()

    @staticmethod
    def batch(callback: Callable[[Patch], None]) -> Dict[str, Any]:
        """Starts a batch operation, executes the callback, and returns the dict response."""
        p = Patch.create()
        callback(p)
        return p.to_dict()

    @staticmethod
    def parse_event(body: Dict[str, Any]) -> Dict[str, Any]:
        """Parses a raw NHTML event from the request body."""
        payload = body.get('payload', '')
        data = {}
        try:
            if payload:
                data = json.loads(payload)
        except json.JSONDecodeError:
            pass
            
        return {
            'handler': body.get('handler'),
            'source_id': body.get('source_id'),
            'session_id': body.get('session_id'),
            'payload': payload,
            'data': data,
            'last_version': body.get('last_version', 0),
            'nodes': body.get('nodes', {})
        }

    @staticmethod
    def join_room(room_id: str) -> Dict[str, Any]:
        """Utility to join a room immediately."""
        return Patch.create().join_room(room_id).to_dict()

    @staticmethod
    def leave_room(room_id: str) -> Dict[str, Any]:
        """Utility to leave a room immediately."""
        return Patch.create().leave_room(room_id).to_dict()
