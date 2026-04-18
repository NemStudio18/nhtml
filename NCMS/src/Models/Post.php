<?php
namespace App\Models;

use App\Core\Database;
use PDO;

class Post {
    public static function all(): array {
        $db = Database::getInstance();
        return $db->query("SELECT p.*, c.name as category_name FROM posts p LEFT JOIN categories c ON p.category_id = c.id ORDER BY p.created_at DESC")->fetchAll();
    }

    public static function allByType(string $type): array {
        $db = Database::getInstance();
        $stmt = $db->prepare("SELECT p.*, c.name as category_name FROM posts p LEFT JOIN categories c ON p.category_id = c.id WHERE p.type = ? ORDER BY p.created_at DESC");
        $stmt->execute([$type]);
        return $stmt->fetchAll();
    }

    public static function find(int $id): ?array {
        $db = Database::getInstance();
        $stmt = $db->prepare("SELECT * FROM posts WHERE id = ?");
        $stmt->execute([$id]);
        $row = $stmt->fetch();
        return $row ?: null;
    }

    public static function create(array $data): bool {
        $db = Database::getInstance();
        $stmt = $db->prepare("INSERT INTO posts (title, content, status, type, category_id, allow_comments, form_id) VALUES (?, ?, ?, ?, ?, ?, ?)");
        return $stmt->execute([
            $data['title'],
            $data['content'],
            $data['status'] ?? 'published',
            $data['type'] ?? 'post',
            $data['category_id'] ?? null,
            $data['allow_comments'] ?? 1,
            $data['form_id'] ?? null
        ]);
    }

    public static function update(int $id, array $data): bool {
        $db = Database::getInstance();
        $stmt = $db->prepare("UPDATE posts SET title = ?, content = ?, status = ?, type = ?, category_id = ?, allow_comments = ?, form_id = ? WHERE id = ?");
        return $stmt->execute([
            $data['title'],
            $data['content'],
            $data['status'],
            $data['type'] ?? 'post',
            $data['category_id'] ?? null,
            $data['allow_comments'] ?? 1,
            $data['form_id'] ?? null,
            $id
        ]);
    }

    public static function delete(int $id): bool {
        $db = Database::getInstance();
        $stmt = $db->prepare("DELETE FROM posts WHERE id = ?");
        return $stmt->execute([$id]);
    }
}
