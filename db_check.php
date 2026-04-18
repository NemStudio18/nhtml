<?php
try {
    $absPath = realpath(__DIR__ . '/NCMS/database.sqlite');
    if (!$absPath) {
        throw new Exception("File not found at " . __DIR__ . '/NCMS/database.sqlite');
    }
    
    $db = new PDO('sqlite:' . $absPath);
    $db->setAttribute(PDO::ATTR_ERRMODE, PDO::ERRMODE_EXCEPTION);

    $posts = $db->query("SELECT COUNT(*) FROM posts")->fetchColumn();
    // On ne vérifie pas users pour l'instant si vous n'êtes pas sûr du nom, mais vérifions les tables
    $tables = $db->query("SELECT name FROM sqlite_master WHERE type='table'")->fetchAll(PDO::FETCH_COLUMN);

    echo "--- DATABASE CHECK ---\n";
    echo "Found Tables: " . implode(", ", $tables) . "\n";
    echo "Total Posts: " . $posts . "\n";

    if ($posts > 0) {
        $lastPosts = $db->query("SELECT title, status FROM posts ORDER BY created_at DESC LIMIT 5")->fetchAll(PDO::FETCH_ASSOC);
        echo "\nLast 5 Posts:\n";
        foreach ($lastPosts as $p) {
            echo "- [" . $p['status'] . "] " . $p['title'] . "\n";
        }
    }
} catch (Exception $e) {
    echo "DATABASE ERROR: " . $e->getMessage() . "\n";
}
