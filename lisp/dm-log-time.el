;;; dm-log-time.el --- Fictitious time handling  -*- lexical-binding: t; -*-

;;; Commentary:
;; Parsing and calculation of game time for logbooks.
;; Supports periods like 10m, 1h, 1d, 2w (weeks).

;;; Code:

(require 'cl-lib)
(require 'parse-time)

;; -----------------------------------------------------------------------------
;; Period parsing
;; -----------------------------------------------------------------------------

(defun dm-log-time--parse-period (str)
  "Parse period string STR (e.g: \"10m\", \"2h\", \"1d\") into seconds.
Return nil if invalid."
  (when (string-match "^\\([0-9]+\\)\\([smhdw]\\)$" str)
    (let ((n (string-to-number (match-string 1 str)))
          (unit (match-string 2 str)))
      (* n
         (pcase unit
           ("s" 1)
           ("m" 60)
           ("h" 3600)
           ("d" 86400)
           ("w" 604800)
           (_ 0))))))

(defun dm-log-time--period-to-string (seconds)
  "Convert SECONDS to readable representation (e.g: 1h, 30m)."
  (cond
   ((>= seconds 604800)
    (format "%dw" (/ seconds 604800)))
   ((>= seconds 86400)
    (format "%dd" (/ seconds 86400)))
   ((>= seconds 3600)
    (format "%dh" (/ seconds 3600)))
   ((>= seconds 60)
    (format "%dm" (/ seconds 60)))
   (t
    (format "%ds" seconds))))

;; -----------------------------------------------------------------------------
;; Game time
;; -----------------------------------------------------------------------------

(defun dm-log-time--parse-game-timestamp (str)
  "Parse game timestamp STR [YYYY-MM-DD HH:MM] to Emacs time.
Return nil if invalid."
  (when (string-match "\\[\\([0-9]\\{4\\}\\)-\\([0-9]\\{2\\}\\)-\\([0-9]\\{2\\}\\) \\([0-9]\\{2\\}\\):\\([0-9]\\{2\\}\\)\\]" str)
    (let ((year (string-to-number (match-string 1 str)))
          (mon (string-to-number (match-string 2 str)))
          (day (string-to-number (match-string 3 str)))
          (hour (string-to-number (match-string 4 str)))
          (min (string-to-number (match-string 5 str))))
      (encode-time 0 min hour day mon year))))

(defun dm-log-time--format-game-timestamp (time &optional fmt)
  "Format TIME (encode-time) using format FMT.
If FMT is nil, use standard format [YYYY-MM-DD HH:MM]."
  (let ((decoded (decode-time time)))
    (if fmt
        (format-time-string fmt time)
      (format "[%04d-%02d-%02d %02d:%02d]"
              (decoded-time-year decoded)
              (decoded-time-month decoded)
              (decoded-time-day decoded)
              (decoded-time-hour decoded)
              (decoded-time-minute decoded)))))

(defun dm-log-time--add-period (time period-str)
  "Add PERIOD-STR to TIME (encode-time). Return new encode-time."
  (let ((seconds (dm-log-time--parse-period period-str)))
    (when seconds
      (time-add time (seconds-to-time seconds)))))

(provide 'dm-log-time)
;;; dm-log-time.el ends here