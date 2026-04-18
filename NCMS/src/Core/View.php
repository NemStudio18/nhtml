<?php
namespace App\Core;

class View {
    private static string $templatesDir = __DIR__ . '/../../templates/';
    private static string $cacheDir = __DIR__ . '/../../public/cache/';
    private static string $transpilerPath = __DIR__ . '/../../../nhtml.py';

    public static function render(string $template, array $data = []): void {
        $sourcePath = self::$templatesDir . $template . '.nhtml';
        $cachePath = self::$cacheDir . $template . '.html';

        if (!file_exists($sourcePath)) {
            die("Erreur : Template introuvable ($template.nhtml)");
        }

        // On-the-fly transpilation if source is newer or cache missing
        if (!file_exists($cachePath) || filemtime($sourcePath) > filemtime($cachePath)) {
            self::transpile($sourcePath, $cachePath);
        }

        $html = file_get_contents($cachePath);

        // Injection des données PHP dans le state Nhtml
        $html = self::injectState($html, $data);

        echo $html;
    }

    private static function transpile(string $source, string $target): void {
        $config = json_decode(file_get_contents(__DIR__ . '/../../config.json'), true);
        $pythonCmds = isset($config['python_path']) ? [$config['python_path']] : ['python', 'python3', 'py'];
        
        $success = false;
        $outputs = [];

        foreach ($pythonCmds as $cmd) {
            $command = escapeshellcmd($cmd) . " " . 
                       escapeshellarg(self::$transpilerPath) . " " . 
                       escapeshellarg($source) . " " . 
                       escapeshellarg($target) . " 2>&1";
            
            $output = shell_exec($command);
            $outputs[$cmd] = $output;

            if (file_exists($target)) {
                $success = true;
                break;
            }
        }

        if (!$success) {
            $errorMsg = "Erreur de transpilation : Impossible de générer $target. <br>";
            $errorMsg .= "Commandes testées : " . implode(', ', $pythonCmds) . "<br>";
            $errorMsg .= "Dernière sortie console : <pre>" . (end($outputs) ?: 'Aucune sortie') . "</pre>";
            die($errorMsg);
        }
    }

    private static function injectState(string $html, array $data): string {
        $jsonState = json_encode($data, JSON_UNESCAPED_UNICODE | JSON_HEX_TAG | JSON_HEX_AMP | JSON_HEX_APOS | JSON_HEX_QUOT);
        
        $mergeScript = "\n<script>
        console.log('[NCMS] Injected State:', $jsonState);
        if (typeof nhtml !== 'undefined') { 
            Object.assign(nhtml, $jsonState); 
        } else { 
            Object.assign(_nhtmlState, $jsonState); 
        }
        </script>";
        
        return str_replace('</body>', $mergeScript . "\n</body>", $html);
    }
}
