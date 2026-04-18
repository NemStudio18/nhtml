<?php
namespace App\Models;

use App\Core\Database;
use PDO;

class Comment {
    public static function forPost(int $postId): array {
        $db = Database::getInstance();
        $stmt = $db->prepare("SELECT * FROM comments WHERE post_id = ? ORDER BY created_at ASC");
        $stmt->execute([$postId]);
        return $stmt->fetchAll();
    }

    public static function create(array $data): bool {
        $db = Database::getInstance();
        $stmt = $db->prepare("INSERT INTO comments (post_id, author, content) VALUES (?, ?, ?)");
        return $stmt->execute([
            $data['post_id'],
            $data['author'],
            $data['content']
        ]);
    }
}
