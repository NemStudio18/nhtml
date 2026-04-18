<?php
namespace App\Core;

class Logger {
    private static string $logDir = __DIR__ . '/../../logs/';

    public static function log(string $message, string $level = 'INFO'): void {
        if (!is_dir(self::$logDir)) {
            mkdir(self::$logDir, 0777, true);
        }

        $date = date('Y-m-d');
        $time = date('H:i:s');
        $file = self::$logDir . $date . '.log';
        
        $ip = $_SERVER['REMOTE_ADDR'] ?? 'CLI';
        $formattedMessage = "[$time] [$level] [$ip] $message" . PHP_EOL;
        
        file_put_contents($file, $formattedMessage, FILE_APPEND);
    }

    public static function info(string $message): void { self::log($message, 'INFO'); }
    public static function warning(string $message): void { self::log($message, 'WARNING'); }
    public static function error(string $message): void { self::log($message, 'ERROR'); }
}
