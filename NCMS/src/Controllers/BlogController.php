<?php
namespace App\Controllers;

use App\Core\View;
use App\Models\Post;
use App\Models\Comment;
use App\Models\Category;
use App\Models\MenuLink;
use App\Models\Form;
use App\Models\FormSubmission;

class BlogController {
    private function getCommonData(): array {
        return [
            'menu_tree'  => MenuLink::getTree(), // Arborescence dynamique
            'categories' => Category::all(),
            'site_name'  => 'NCMS Blog',
            'is_admin'   => \App\Core\Auth::check()
        ];
    }

    public function index(): void {
        $posts = Post::allByType('post');
        $published = array_filter($posts, fn($p) => $p['status'] === 'published');
        
        View::render('blog', array_merge($this->getCommonData(), [
            'posts' => array_values($published)
        ]));
    }

    public function show(array $params): void {
        $post = Post::find($params['id']);
        if (!$post || $post['status'] !== 'published') {
            http_response_code(404); echo "Article non trouvé"; return;
        }

        $comments = Comment::forPost($params['id']);
        $form     = $post['form_id'] ? Form::find($post['form_id']) : null;

        View::render('post', array_merge($this->getCommonData(), [
            'post' => $post,
            'comments' => $comments,
            'attached_form' => $form
        ]));
    }

    public function showPage(array $params): void {
        $page = Post::find($params['id']);
        if (!$page || $page['type'] !== 'page' || $page['status'] !== 'published') {
            http_response_code(404); echo "Page non trouvée"; return;
        }

        $form = $page['form_id'] ? Form::find($page['form_id']) : null;

        View::render('post', array_merge($this->getCommonData(), [
            'post' => $page,
            'attached_form' => $form
        ]));
    }

    public function submitForm(): void {
        $formId = (int)($_POST['form_id'] ?? 0);
        $data   = $_POST;
        unset($data['form_id']);

        if ($formId) {
            FormSubmission::create($formId, $data);
        }

        header('Location: ' . ($_SERVER['HTTP_REFERER'] ?: '/') . '?success=1');
        exit;
    }

    public function category(array $params): void {
        $catId = (int)$params['id'];
        $category = Category::find($catId);
        if (!$category) {
            http_response_code(404); echo "Catégorie non trouvée"; return;
        }

        $posts = Post::allByType('post');
        $filtered = array_filter($posts, fn($p) => (int)$p['category_id'] === $catId && $p['status'] === 'published');

        View::render('blog', array_merge($this->getCommonData(), [
            'posts' => array_values($filtered),
            'current_category' => $category['name']
        ]));
    }
}

