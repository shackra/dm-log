;;; dm-log-org.el --- Org-mode parser and writer  -*- lexical-binding: t; -*-

;;; Commentary:
;; Functions for reading and writing org-mode structure of logbooks,
;; players, consumables, and game time.

;;; Code:

(require 'cl-lib)
(require 'org)
(require 'org-element)
(require 'dm-log-time)

;; -----------------------------------------------------------------------------
;; File properties (campaign)
;; -----------------------------------------------------------------------------

(defun dm-log-org--get-file-properties (file)
  "Read top-level file properties from FILE.
Return plist with :time-format, :current-time, etc."
  (when (file-exists-p file)
    (with-temp-buffer
      (insert-file-contents file)
      (org-mode)
      (org-element-map (org-element-parse-buffer) 'property-drawer
        (lambda (pd)
          (let ((props (org-element-property :properties pd)))
            (when props
              (let ((fmt (or (cdr (assoc "TIME_FORMAT" props))
                             (cdr (assoc "FORMATO_TIEMPO" props))))
                    (time (or (cdr (assoc "CURRENT_TIME" props))
                              (cdr (assoc "TIEMPO_ACTUAL" props)))))
                (list :time-format (or fmt "%B %d, %E %Y %H:%M")
                      :current-time time)))))
        nil t))))

(defun dm-log-org--set-file-property (file prop value)
  "Set property PROP to VALUE in FILE."
  (when (file-exists-p file)
    (with-current-buffer (find-file-noselect file)
      (save-excursion
        (goto-char (point-min))
        (if (re-search-forward (concat "^#\\+\\(" prop "\\):\\s-*\\(.*\\)$") nil t)
            (replace-match (concat "#+" prop ": " value))
          (goto-char (point-min))
          (insert "#+" prop ": " value "\n"))
        (save-buffer))
      (kill-buffer (current-buffer)))))

(defun dm-log-org--update-current-time (file time &optional fmt)
  "Update CURRENT_TIME in FILE.
If FMT is non-nil, also update TIME_FORMAT."
  (with-current-buffer (find-file-noselect file)
    (save-excursion
      (goto-char (point-min))
      (let ((time-str (dm-log-time--format-game-timestamp time)))
        ;; Try new property name first, then fall back to old name
        (if (re-search-forward "^:CURRENT_TIME:" nil t)
            (progn
              (beginning-of-line)
              (kill-line)
              (insert ":CURRENT_TIME: " time-str))
          (if (re-search-forward "^:TIEMPO_ACTUAL:" nil t)
              (progn
                (beginning-of-line)
                (kill-line)
                (insert ":CURRENT_TIME: " time-str))
            (progn
              (goto-char (point-min))
              (when (re-search-forward "^:END:" nil t)
                (end-of-line)
                (insert "\n:CURRENT_TIME: " time-str)
                (insert "\n:END:"))))))
      (when fmt
        (goto-char (point-min))
        (if (re-search-forward "^:TIME_FORMAT:" nil t)
            (progn
              (beginning-of-line)
              (kill-line)
              (insert ":TIME_FORMAT: " fmt))
          (if (re-search-forward "^:FORMATO_TIEMPO:" nil t)
              (progn
                (beginning-of-line)
                (kill-line)
                (insert ":TIME_FORMAT: " fmt))
            (progn
              (goto-char (point-min))
              (when (re-search-forward "^:END:" nil t)
                (end-of-line)
                (insert "\n:TIME_FORMAT: " fmt)
                (insert "\n:END:"))))))
      (save-buffer))
    (kill-buffer (current-buffer))))

;; -----------------------------------------------------------------------------
;; Players
;; -----------------------------------------------------------------------------

(defun dm-log-org--get-players (file)
  "Read players from FILE. Return list of plists:
(:name N :props PLIST :inventory ALIST)."
  (when (file-exists-p file)
    (with-temp-buffer
      (insert-file-contents file)
      (org-mode)
      (let (players)
        (org-element-map (org-element-parse-buffer) 'headline
          (lambda (hl)
            (when (= (org-element-property :level hl) 1)
              (let* ((name (org-element-property :raw-value hl))
                     (props (org-element-property :properties hl))
                     (inventory nil))
                (dolist (p props)
                  (let ((k (car p))
                        (v (cdr p)))
                    (when (and v (not (member k '("ID" "NOMBRE" "HERO" "ANTORCHA_ENCENDIDA" "TORCH-LIT"))))
                      (let ((n (string-to-number v)))
                        (when (> n 0)
                          (push (cons k n) inventory))))))
                (push (list :name name
                            :props (list :torch-lit (string= (or (cdr (assoc "TORCH-LIT" props))
                                                                  (cdr (assoc "ANTORCHA_ENCENDIDA" props))) "t"))
                            :inventory (nreverse inventory))
                      players)))))
        (nreverse players)))))

;; -----------------------------------------------------------------------------
;; Logbook - Entries
;; -----------------------------------------------------------------------------

(defun dm-log-org--get-notes (pos)
  "Extract text from the Notes sub-heading at POS."
  (save-excursion
    (goto-char pos)
    (when (re-search-forward "^\\*\\*\\*\\*\\s-+Notes" nil t)
      (let ((start (line-end-position)))
        (if (re-search-forward "^\\*" nil t)
            (buffer-substring-no-properties start (line-beginning-position))
          (buffer-substring-no-properties start (point-max)))))))

(defun dm-log-org--get-last-entry (file)
  "Get the last turn entry from FILE.
Return plist with turn data."
  (when (file-exists-p file)
    (with-temp-buffer
      (insert-file-contents file)
      (org-mode)
      (goto-char (point-max))
      (let ((found nil))
        (while (and (not found) (re-search-backward "^\\*\\*\\*\\s-+" nil t))
          (let ((props (org-entry-properties)))
            (when (or (assoc "TURN_NUMBER" props) (assoc "TURNO_NUMERO" props))
              (setq found (list
                           :number (string-to-number (or (cdr (assoc "TURN_NUMBER" props))
                                                         (cdr (assoc "TURNO_NUMERO" props))))
                           :type (or (cdr (assoc "TURN_TYPE" props))
                                     (cdr (assoc "TURNO_TIPO" props)))
                           :time-start (or (cdr (assoc "TIME_START" props))
                                           (cdr (assoc "TIEMPO_INICIO" props)))
                           :time-end (or (cdr (assoc "TIME_END" props))
                                        (cdr (assoc "TIEMPO_FIN" props)))
                           :advance (or (cdr (assoc "ADVANCE" props))
                                       (cdr (assoc "AVANCE" props)))
                           :notes (dm-log-org--get-notes (point))
                           :consumables (dm-log-org--get-consumables-table (point))
                           :players (or (cdr (assoc "ACTIVE_PLAYERS" props))
                                        (cdr (assoc "JUGADORES_ACTIVOS" props))))))))
        found))))

(defun dm-log-org--get-consumables-table (pos)
  "Extract consumables table from POS.
Return alist: ((player . ((item . quantity)...)) ...)"
  (save-excursion
    (goto-char pos)
    (let (result)
      (when (re-search-forward "^\\*\\*\\*\\*\\s-+Consumables" nil t)
        (forward-line)
        (when (looking-at "^\\s-*|")
          (forward-line 2))
        (while (looking-at "^\\s-*|\\s-*\\([^|]+\\)|")
          (let ((item (string-trim (match-string 1)))
                (cols nil))
            (save-excursion
              (beginning-of-line)
              (while (re-search-forward "|\\s-*\\([^|]+\\)" (line-end-position) t)
                (push (string-trim (match-string 1)) cols)))
            (push (cons item (nreverse (cdr (nreverse cols)))) result))
          (forward-line)))
      result)))

;; -----------------------------------------------------------------------------
;; Insert entry
;; -----------------------------------------------------------------------------

(defun dm-log-org--insert-turn (file number type advance notes consumables-alist _players-inventory time-start time-end)
  "Insert new turn entry in FILE.
CONSUMABLES-ALIST: ((player . ((item . quantity)...)) ...)
PLAYERS-INVENTORY is reserved for future use."
  (with-current-buffer (find-file-noselect file)
    (save-excursion
      (goto-char (point-max))
      (insert (format "\n*** Turn %d [%s] (%s)\n" number (capitalize type) advance))
      (insert ":PROPERTIES:\n")
      (insert (format ":TURN_NUMBER: %d\n" number))
      (insert (format ":TURN_TYPE: %s\n" type))
      (insert (format ":TIME_START: %s\n" (dm-log-time--format-game-timestamp time-start)))
      (insert (format ":TIME_END: %s\n" (dm-log-time--format-game-timestamp time-end)))
      (insert (format ":ADVANCE: %s\n" advance))
      (insert ":END:\n\n")
      (insert "**** Notes\n")
      (insert notes "\n\n")
      (insert "**** Consumables\n")
      (let* ((names (mapcar #'car consumables-alist))
             (num-players (length names)))
        (insert "| Item |")
        (dolist (name names)
          (insert (format " %s |" name)))
        (insert "\n")
        (insert "|------")
        (dotimes (_ num-players)
          (insert "+------"))
        (insert "|\n")
        (let* ((items (delete-dups
                       (apply #'append
                              (mapcar (lambda (j) (mapcar #'car (cdr j)))
                                      consumables-alist)))))
          (dolist (item items)
            (insert (format "| %s |" item))
            (dolist (player names)
              (let ((quantity (or (cdr (assoc-string item (cdr (assoc-string player consumables-alist)))) 0)))
                (insert (format " %.2f |" quantity))))
            (insert "\n"))))
      (insert "\n")
      (save-buffer))
    (kill-buffer (current-buffer))))

(defun dm-log-org--insert-time-jump (file notes time-start time-end advance)
  "Insert time jump entry."
  (with-current-buffer (find-file-noselect file)
    (save-excursion
      (goto-char (point-max))
      (insert (format "\n*** Time jump: %s (%s)\n" notes advance))
      (insert ":PROPERTIES:\n")
      (insert ":ENTRY_TYPE: time_jump\n")
      (insert (format ":TIME_START: %s\n" (dm-log-time--format-game-timestamp time-start)))
      (insert (format ":TIME_END: %s\n" (dm-log-time--format-game-timestamp time-end)))
      (insert (format ":ADVANCE: %s\n" advance))
      (insert ":END:\n\n")
      (insert "**** Notes\n")
      (insert notes "\n\n")
      (save-buffer))
    (kill-buffer (current-buffer))))

(provide 'dm-log-org)
;;; dm-log-org.el ends here