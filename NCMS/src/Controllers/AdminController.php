<?php
namespace App\Controllers;

use App\Core\Auth;
use App\Core\View;
use App\Models\Post;
use App\Models\Category;
use App\Models\MenuLink;
use App\Models\Form;
use App\Models\FormSubmission;
use App\Core\NotFoundException;

class AdminController {
    public function __construct() {
        Auth::requireAdmin();
    }

    private function baseData(): array {
        return [
            'posts'         => Post::all(),
            'categories'    => Category::all(),
            'menu_links'    => MenuLink::all(),
            'forms'         => Form::all(),
            'submissions'   => FormSubmission::all(),
            'site_name'     => 'NCMS Admin',
        ];
    }

    public function dashboard(): void {
        $view = $_GET['view'] ?? 'articles';
        if (!in_array($view, ['articles', 'pages', 'categories', 'navigation', 'forms', 'submissions'])) {
            $view = 'articles';
        }

        View::render('admin', array_merge($this->baseData(), [
            'current_view' => $view,
            'editing_post' => ['id' => null, 'title' => '', 'content' => '', 'status' => 'published', 'type' => 'post', 'category_id' => null, 'allow_comments' => 1, 'form_id' => null],
        ]));
    }

    public function create(): void {
        View::render('admin', array_merge($this->baseData(), [
            'current_view' => 'new',
            'editing_post' => ['id' => null, 'title' => '', 'content' => '', 'status' => 'published', 'type' => 'post', 'category_id' => null, 'allow_comments' => 1, 'form_id' => null],
        ]));
    }

    public function edit(array $params): void {
        $id   = (int)($params['id'] ?? 0);
        $post = Post::find($id);

        if (!$post) {
            throw new NotFoundException("Post $id introuvable");
        }

        View::render('admin', array_merge($this->baseData(), [
            'current_view' => 'edit',
            'editing_post' => $post,
        ]));
    }

    public function save(): void {
        $id   = $_POST['id'] ?? null;
        $data = [
            'title'          => $_POST['title'] ?? '',
            'content'        => $_POST['content'] ?? '',
            'status'         => $_POST['status'] ?? 'published',
            'type'           => $_POST['type'] ?? 'post',
            'category_id'    => !empty($_POST['category_id']) ? (int)$_POST['category_id'] : null,
            'allow_comments' => isset($_POST['allow_comments']) ? (int)$_POST['allow_comments'] : 1,
            'form_id'        => !empty($_POST['form_id']) ? (int)$_POST['form_id'] : null,
        ];

        if ($id && is_numeric($id)) {
            Post::update((int)$id, $data);
        } else {
            Post::create($data);
        }

        $redirect = ($data['type'] === 'page') ? '/admin?view=pages' : '/admin?view=articles';
        header('Location: ' . $redirect);
        exit;
    }

    public function delete(): void {
        $id   = $_POST['id'] ?? null;
        $type = $_POST['type'] ?? 'post';
        if ($id) {
            Post::delete((int)$id);
        }
        $redirect = ($type === 'page') ? '/admin?view=pages' : '/admin?view=articles';
        header('Location: ' . $redirect);
        exit;
    }

    // --- CATEGORIES ---
    public function saveCategory(): void {
        $id   = $_POST['id'] ?? null;
        $name = $_POST['name'] ?? '';
        $slug = $_POST['slug'] ?? '';
        $parent_id = !empty($_POST['parent_id']) ? (int)$_POST['parent_id'] : null;
        
        if ($name) {
            $db = \App\Core\Database::getInstance();
            if (empty($slug)) {
                $slug = strtolower(str_replace(' ', '-', $name));
            }
            if ($id && is_numeric($id)) {
                $stmt = $db->prepare("UPDATE categories SET name = ?, slug = ?, parent_id = ? WHERE id = ?");
                $stmt->execute([$name, $slug, $parent_id, (int)$id]);
            } else {
                $stmt = $db->prepare("INSERT INTO categories (name, slug, parent_id) VALUES (?, ?, ?)");
                $stmt->execute([$name, $slug, $parent_id]);
            }
        }
        header('Location: /admin?view=categories');
        exit;
    }

    public function deleteCategory(): void {
        $id = $_POST['id'] ?? null;
        if ($id) {
            $db = \App\Core\Database::getInstance();
            $stmt = $db->prepare("DELETE FROM categories WHERE id = ?");
            $stmt->execute([(int)$id]);
        }
        header('Location: /admin?view=categories');
        exit;
    }

    // --- NAVIGATION ---
    public function saveMenu(): void {
        $id    = $_POST['id'] ?? null;
        $data = [
            'label'     => $_POST['label'] ?? '',
            'url'       => $_POST['url'] ?? '/',
            'position'  => (int)($_POST['position'] ?? 0),
            'parent_id' => !empty($_POST['parent_id']) ? (int)$_POST['parent_id'] : null
        ];

        if ($id && is_numeric($id)) {
            MenuLink::update((int)$id, $data);
        } else {
            MenuLink::create($data);
        }
        header('Location: /admin?view=navigation');
        exit;
    }

    public function deleteMenu(): void {
        $id = $_POST['id'] ?? null;
        if ($id) {
            MenuLink::delete((int)$id);
        }
        header('Location: /admin?view=navigation');
        exit;
    }

    // --- FORMS ---
    public function saveForm(): void {
        $id   = $_POST['id'] ?? null;
        $name = $_POST['name'] ?? '';
        
        // Simple processing of fields from textarea (comma separated for demo)
        $fieldsRaw = $_POST['fields_raw'] ?? '';
        $fields = [];
        foreach (explode("\n", $fieldsRaw) as $line) {
            $parts = explode(':', trim($line));
            if (count($parts) >= 2) {
                $fields[] = ['label' => $parts[0], 'type' => $parts[1]];
            }
        }

        $data = ['name' => $name, 'fields' => $fields];

        if ($id && is_numeric($id)) {
            Form::update((int)$id, $data);
        } else {
            Form::create($data);
        }
        header('Location: /admin?view=forms');
        exit;
    }

    public function deleteForm(): void {
        $id = $_POST['id'] ?? null;
        if ($id) {
            Form::delete((int)$id);
        }
        header('Location: /admin?view=forms');
        exit;
    }

}
