import sys
import os
sys.path.append(os.path.abspath("sdk/python"))
from nhtml.patch import Patch

def test_patch_operations():
    patch = Patch.create()
    patch.set_text("t1", "Hello").add_class("t1", "active").set_style("t1", "color", "red")
    
    data = patch.to_dict()
    assert len(data['patch']) == 3
    assert data['patch'][0]['op'] == 'set_text'
    assert data['patch'][1]['op'] == 'add_class'
    assert data['patch'][2]['op'] == 'set_style'
    print("test_patch_operations: PASSED")

def test_rooms():
    patch = Patch.create()
    patch.join_room("chat_123").leave_room("lobby")
    
    data = patch.to_dict()
    assert "chat_123" in data['join_room']
    assert "lobby" in data['leave_room']
    print("test_rooms: PASSED")

if __name__ == "__main__":
    test_patch_operations()
    test_rooms()
    print("All Python SDK unit tests PASSED")
