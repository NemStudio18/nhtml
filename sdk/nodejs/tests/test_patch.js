const { Patch, parseEvent } = require('../index');
const assert = require('assert');

function testPatchOperations() {
    const patch = Patch.create();
    patch.setText("t1", "Hello").addClass("t1", "active").setStyle("t1", "color", "red");
    
    const data = patch.toDict();
    assert.strictEqual(data.patch.length, 3);
    assert.strictEqual(data.patch[0].op, 'set_text');
    assert.strictEqual(data.patch[1].op, 'add_class');
    assert.strictEqual(data.patch[2].op, 'set_style');
    console.log("testPatchOperations: PASSED");
}

function testRooms() {
    const patch = Patch.create();
    patch.joinRoom("chat_123").leaveRoom("lobby");
    
    const data = patch.toDict();
    assert.ok(data.join_room.includes("chat_123"));
    assert.ok(data.leave_room.includes("lobby"));
    console.log("testRooms: PASSED");
}

testPatchOperations();
testRooms();
console.log("All Node.js SDK unit tests PASSED");
