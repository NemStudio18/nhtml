<?php

namespace Nhtml;

class NhtmlCompiler {
    private static $ffi = null;
    private static $initialized = false;
    private static $lib_path = "";
    private static $bin_path = "";

    /**
     * Initialise le compilateur (tente de charger le `.dll` ou `.so`)
     */
    public static function init() {
        if (self::$initialized) return;

        $dir = __DIR__ . '/../nhtml-core/target/debug';
        
        // Détection de l'OS pour charger la bonne lib dynamique
        if (strtoupper(substr(PHP_OS, 0, 3)) === 'WIN') {
            self::$lib_path = $dir . '/nhtml_core.dll';
            self::$bin_path = $dir . '/nhtml.exe';
        } else {
            // Linux ou macOS (.so / .dylib)
            self::$lib_path = $dir . '/libnhtml_core.so'; // fallback binaire non testé pour Mac ici
            if (!file_exists(self::$lib_path) && file_exists($dir . '/libnhtml_core.dylib')) {
                self::$lib_path = $dir . '/libnhtml_core.dylib';
            }
            self::$bin_path = $dir . '/nhtml';
        }

        // Tenter de charger FFI
        if (extension_loaded('ffi') && file_exists(self::$lib_path)) {
            try {
                // Définition de l'interface C attendue
                self::$ffi = \FFI::cdef("
                    char* nhtml_compile(const char* input);
                    void nhtml_free(char* s);
                ", self::$lib_path);
            } catch (\FFI\Exception $e) {
                error_log("[Nhtml] FFI Init Error: " . $e->getMessage());
                self::$ffi = null;
            }
        }

        self::$initialized = true;
    }

    /**
     * Compile le fichier ou le contenu Nhtml.
     * Privilégie FFI (Ultra-Rapide), fallback sur exec() (Mode Mutualisé).
     */
    public static function compile(string $nhtml_source): array {
        self::init();

        // 1. TENTATIVE FFI (Exécution en RAM sans processus)
        if (self::$ffi !== null) {
            // Passer la chaine en mémoire C
            $c_str = self::$ffi->nhtml_compile($nhtml_source);
            
            if (!\FFI::isNull($c_str)) {
                // Récupérer le retour
                $json_str = \FFI::string($c_str);
                
                // Libérer la mémoire gérée par Rust pour éviter les fuites !
                self::$ffi->nhtml_free($c_str);
                
                $decoded = json_decode($json_str, true);
                if ($decoded) {
                    return [
                        'html' => $decoded['html'] ?? '',
                        'manifest' => $decoded['manifest'] ?? [],
                        'mode' => 'FFI'
                    ];
                }
            }
        }

        // 2. FALLBACK EXEC (Slower execution)
        if (file_exists(self::$bin_path)) {
            // Créer un fichier temporaire pour passer la source au binaire
            $tmp_in = tempnam(sys_get_temp_dir(), 'nhtml_in_');
            $tmp_out = tempnam(sys_get_temp_dir(), 'nhtml_out_');
            
            file_put_contents($tmp_in, $nhtml_source);
            
            // exec() bloque jusqu'à la fin de l'exécution
            exec(escapeshellarg(self::$bin_path) . " " . escapeshellarg($tmp_in) . " " . escapeshellarg($tmp_out));
            
            $html = '';
            $manifest = [];
            
            if (file_exists($tmp_out . '.html')) {
                $html = file_get_contents($tmp_out . '.html');
                unlink($tmp_out . '.html');
            }
            if (file_exists($tmp_out . '.json')) {
                $manifest = json_decode(file_get_contents($tmp_out . '.json'), true);
                unlink($tmp_out . '.json');
            }
            
            unlink($tmp_in);
            
            return [
                'html' => $html,
                'manifest' => $manifest,
                'mode' => 'CLI'
            ];
        }

        throw new \Exception("NhtmlCompiler: FFI indisponible et exécutable non trouvé (" . self::$bin_path . ")");
    }
}
