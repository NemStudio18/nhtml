<?php
$db = new PDO('sqlite:NCMS/database/database.sqlite');
$res = $db->query("SELECT * FROM categories")->fetchAll(PDO::FETCH_ASSOC);
echo json_encode($res, JSON_PRETTY_PRINT);
