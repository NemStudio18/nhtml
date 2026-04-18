<?php
namespace App\Models;

use App\Core\Database;
use PDO;

class FormSubmission {
    public static function all(): array {
        $db = Database::getInstance();
        return $db->query("SELECT s.*, f.name as form_name FROM form_submissions s LEFT JOIN forms f ON s.form_id = f.id ORDER BY s.created_at DESC")->fetchAll();
    }

    public static function forForm(int $formId): array {
        $db = Database::getInstance();
        $stmt = $db->prepare("SELECT * FROM form_submissions WHERE form_id = ? ORDER BY created_at DESC");
        $stmt->execute([$formId]);
        return $stmt->fetchAll();
    }

    public static function create(int $formId, array $data): bool {
        $db = Database::getInstance();
        $stmt = $db->prepare("INSERT INTO form_submissions (form_id, data) VALUES (?, ?)");
        return $stmt->execute([
            $formId,
            json_encode($data)
        ]);
    }

    public static function delete(int $id): bool {
        $db = Database::getInstance();
        $stmt = $db->prepare("DELETE FROM form_submissions WHERE id = ?");
        return $stmt->execute([$id]);
    }
}
