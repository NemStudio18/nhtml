<?php
require_once __DIR__ . '/NCMS/src/Core/Database.php';
require_once __DIR__ . '/NCMS/src/Models/MenuLink.php';
use App\Models\MenuLink;

$tree = MenuLink::getTree();
echo json_encode($tree, JSON_PRETTY_PRINT);
