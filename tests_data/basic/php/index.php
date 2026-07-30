<?php

declare(strict_types=1);

function render_row(string $name, int $score): string
{
    return sprintf("<li>%s: %d</li>", htmlspecialchars($name, ENT_QUOTES), $score);
}

$scores = [
    "alpha" => 95,
    "beta" => 88,
    "gamma" => 91,
];
?>
<!doctype html>
<html>
<body>
<ul>
<?php foreach ($scores as $name => $score): ?>
    <?= render_row($name, $score) ?>
<?php endforeach ?>
</ul>
</body>
</html>
