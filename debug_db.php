<?php
require_once __DIR__ . '/NCMS/src/Core/Database.php';
use App\Core\Database;

try {
    $db = Database::getInstance();
    $rows = $db->query("SELECT * FROM menu_links")->fetchAll(PDO::FETCH_ASSOC);
    echo json_encode($rows, JSON_PRETTY_PRINT);
} catch (Exception $e) {
    echo "Error: " . $e->getMessage();
}
