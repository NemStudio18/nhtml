<?php
namespace App\Models;

use App\Core\Database;
use PDO;

class Form {
    public static function all(): array {
        $db = Database::getInstance();
        return $db->query("SELECT * FROM forms ORDER BY name ASC")->fetchAll();
    }

    public static function find(int $id): ?array {
        $db = Database::getInstance();
        $stmt = $db->prepare("SELECT * FROM forms WHERE id = ?");
        $stmt->execute([$id]);
        $row = $stmt->fetch();
        if ($row) {
            $row['fields'] = json_decode($row['fields'], true);
        }
        return $row ?: null;
    }

    public static function create(array $data): bool {
        $db = Database::getInstance();
        $stmt = $db->prepare("INSERT INTO forms (name, fields) VALUES (?, ?)");
        return $stmt->execute([
            $data['name'],
            json_encode($data['fields'] ?? [])
        ]);
    }

    public static function update(int $id, array $data): bool {
        $db = Database::getInstance();
        $stmt = $db->prepare("UPDATE forms SET name = ?, fields = ? WHERE id = ?");
        return $stmt->execute([
            $data['name'],
            json_encode($data['fields'] ?? []),
            $id
        ]);
    }

    public static function delete(int $id): bool {
        $db = Database::getInstance();
        $stmt = $db->prepare("DELETE FROM forms WHERE id = ?");
        return $stmt->execute([$id]);
    }
}
