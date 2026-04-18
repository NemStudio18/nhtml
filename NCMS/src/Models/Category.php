<?php
namespace App\Models;

use App\Core\Database;
use PDO;

class Category {
    public static function all(): array {
        $db = Database::getInstance();
        return $db->query("SELECT * FROM categories ORDER BY name ASC")->fetchAll();
    }

    public static function find(int $id): ?array {
        $db = Database::getInstance();
        $stmt = $db->prepare("SELECT * FROM categories WHERE id = ?");
        $stmt->execute([$id]);
        $row = $stmt->fetch();
        return $row ?: null;
    }

    public static function create(array $data): bool {
        $db = Database::getInstance();
        $slug = $data['slug'] ?? strtolower(str_replace(' ', '-', $data['name']));
        $stmt = $db->prepare("INSERT INTO categories (name, slug, parent_id) VALUES (?, ?, ?)");
        return $stmt->execute([
            $data['name'],
            $slug,
            $data['parent_id'] ?? null
        ]);
    }

    public static function delete(int $id): bool {
        $db = Database::getInstance();
        $stmt = $db->prepare("DELETE FROM categories WHERE id = ?");
        return $stmt->execute([$id]);
    }
}
