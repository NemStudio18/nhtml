<?php
namespace App\Models;

use App\Core\Database;
use PDO;

class MenuLink {
    public static function all(): array {
        $db = Database::getInstance();
        return $db->query("SELECT * FROM menu_links ORDER BY position ASC")->fetchAll();
    }

    public static function find(int $id): ?array {
        $db = Database::getInstance();
        $stmt = $db->prepare("SELECT * FROM menu_links WHERE id = ?");
        $stmt->execute([$id]);
        $row = $stmt->fetch();
        return $row ?: null;
    }

    public static function create(array $data): bool {
        $db = Database::getInstance();
        $stmt = $db->prepare("INSERT INTO menu_links (label, url, position, parent_id) VALUES (?, ?, ?, ?)");
        return $stmt->execute([
            $data['label'],
            $data['url'],
            $data['position'] ?? 0,
            $data['parent_id'] ?? null
        ]);
    }

    public static function update(int $id, array $data): bool {
        $db = Database::getInstance();
        $stmt = $db->prepare("UPDATE menu_links SET label = ?, url = ?, position = ?, parent_id = ? WHERE id = ?");
        return $stmt->execute([
            $data['label'],
            $data['url'],
            $data['position'],
            $data['parent_id'],
            $id
        ]);
    }

    public static function delete(int $id): bool {
        $db = Database::getInstance();
        $stmt = $db->prepare("DELETE FROM menu_links WHERE id = ?");
        return $stmt->execute([$id]);
    }

    public static function getTree(): array {
        $all = self::all();
        $tree = [];
        $byParent = [];
        
        foreach ($all as $item) {
            $parentId = $item['parent_id'] ?: 0;
            $byParent[$parentId][] = $item;
        }
        
        if (isset($byParent[0])) {
            foreach ($byParent[0] as $item) {
                $item['children'] = $byParent[$item['id']] ?? [];
                $tree[] = $item;
            }
        }
        
        return $tree;
    }
}
