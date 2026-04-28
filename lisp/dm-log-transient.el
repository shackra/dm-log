;;; dm-log-transient.el --- Transient menus for dm-log  -*- lexical-binding: t; -*-

;;; Commentary:
;; Defines main and secondary transients for navigation and editing.

;;; Code:

(require 'cl-lib)
(require 'transient)
(require 'dm-log-time)
(require 'dm-log-consumables)
(require 'dm-log-org)
(require 'dm-log-ui)
(require 'dm-log-campaign)
(require 'dm-log-map)

(defvar dm-log--current-logbook-file nil)
(defvar dm-log--current-players-file nil)
(defvar dm-log--current-consumables-file nil)
(defvar dm-log-turn-dungeon-advance nil)
(defvar dm-log-turn-wilderness-advance nil)
(defvar dm-log-turn-combat-advance nil)
(defvar dm-log--current-campaign nil)

;; -----------------------------------------------------------------------------
;; Main Transient
;; -----------------------------------------------------------------------------

(transient-define-prefix dm-log-main-menu ()
  "Main menu for Sword & Wizardry Logbook."
  ["Logbook"
   ["Actions"
    ("r" "Refresh" dm-log-transient-refresh)
    ("a" "Add entry" dm-log-transient-add-entry)
    ("t" "Time jump" dm-log-transient-time-jump)]
   ["Campaign"
    ("c" "Switch campaign" dm-log-transient-switch-campaign)
    ("m" "Map editor" dm-log-map)
    ("q" "Quit menu" transient-quit-one)]]
  (interactive)
  (if dm-log--current-campaign
      (transient-setup 'dm-log-main-menu)
    (message "No campaign loaded. Run M-x dm-log")))

;; -----------------------------------------------------------------------------
;; Main actions
;; -----------------------------------------------------------------------------

(defun dm-log-transient-refresh ()
  "Refresh the *SW-Logbook* buffer."
  (interactive)
  (when dm-log--current-logbook-file
    (dm-log-campaign--open-logbook)
    (message "Logbook updated.")))

(defun dm-log-transient-switch-campaign ()
  "Switch to another campaign."
  (interactive)
  (dm-log-campaign-select))

;; -----------------------------------------------------------------------------
;; Sub-transient: Add entry
;; -----------------------------------------------------------------------------

(transient-define-prefix dm-log-transient-add-entry ()
  "Select type of entry to add."
  ["Turn type"
   ["Exploration"
    ("d" "Dungeon" dm-log-transient-turn-dungeon)
    ("e" "Wilderness" dm-log-transient-turn-wilderness)]
   ["Combat"
    ("c" "Combat" dm-log-transient-turn-combat)]
   ["Time"
    ("s" "Arbitrary jump" dm-log-transient-time-jump-from-add)
    ("q" "Cancel" transient-quit-one)]])

;; -----------------------------------------------------------------------------
;; Predefined turns
;; -----------------------------------------------------------------------------

(defun dm-log-transient-turn-dungeon ()
  "Add dungeon turn."
  (interactive)
  (dm-log-transient--process-turn "dungeon" dm-log-turn-dungeon-advance))

(defun dm-log-transient-turn-wilderness ()
  "Add wilderness turn."
  (interactive)
  (dm-log-transient--process-turn "wilderness" dm-log-turn-wilderness-advance))

(defun dm-log-transient-turn-combat ()
  "Add combat turn."
  (interactive)
  (dm-log-transient--process-turn "combat" dm-log-turn-combat-advance))

;; -----------------------------------------------------------------------------
;; Arbitrary time jump
;; -----------------------------------------------------------------------------

(defun dm-log-transient-time-jump ()
  "Arbitrary time jump from main menu."
  (interactive)
  (dm-log-transient--time-jump-internal))

(defun dm-log-transient-time-jump-from-add ()
  "Arbitrary time jump from add menu."
  (interactive)
  (dm-log-transient--time-jump-internal))

(defun dm-log-transient--time-jump-internal ()
  "Prompt for period and notes, then insert time jump."
  (let* ((period (read-string "Time to advance (e.g: 2s, 3d, 1m, 4h): "))
         (notes (read-string "Notes for the time jump: "))
         (props (dm-log-org--get-file-properties dm-log--current-logbook-file))
         (current-time-str (plist-get props :current-time))
         (current-time (or (dm-log-time--parse-game-timestamp current-time-str)
                           (encode-time 0 0 12 1 1 2024)))
         (new-time (dm-log-time--add-period current-time period)))
    (when new-time
      (dm-log-org--insert-time-jump dm-log--current-logbook-file
                                    notes
                                    current-time
                                    new-time
                                    period)
      (dm-log-org--update-current-time dm-log--current-logbook-file new-time)
      (dm-log-campaign--open-logbook)
      (message "Time jump added: %s" period))))

;; -----------------------------------------------------------------------------
;; Process complete turn
;; -----------------------------------------------------------------------------

(defun dm-log-transient--process-turn (type advance)
  "Process creation of turn of TYPE with ADVANCE."
  (let* ((props (dm-log-org--get-file-properties dm-log--current-logbook-file))
         (current-time-str (plist-get props :current-time))
         (current-time (or (dm-log-time--parse-game-timestamp current-time-str)
                           (encode-time 0 0 12 1 1 2024)))
         (new-time (dm-log-time--add-period current-time advance))
         (players (dm-log-org--get-players dm-log--current-players-file))
         (rates (dm-log-consumables--read-table dm-log--current-consumables-file))
         (consumption (dm-log-consumables--calculate-consumption rates advance))
         ;; Calculate next turn number
         (last (dm-log-org--get-last-entry dm-log--current-logbook-file))
         (number (if last (1+ (plist-get last :number)) 1)))
    
    ;; Prepare data for edit buffer
    ;; Use last turn's consumables as starting inventory if available,
    ;; otherwise fall back to players.org inventory
    (let* ((last-consumables (when last (plist-get last :consumables)))
           (inventories (mapcar
                         (lambda (j)
                           (let* ((name (plist-get j :name))
                                  (base-inv (or (when last-consumables
                                                  (cdr (assoc-string name last-consumables t)))
                                                (copy-alist (plist-get j :inventory))))
                                  (new-inv (dm-log-consumables--apply-consumption
                                            base-inv consumption (plist-get j :props))))
                             (cons name new-inv)))
                         players))
           (buf (get-buffer-create "*dm-log-edit-turn*")))
      
      (with-current-buffer buf
        (erase-buffer)
        (org-mode)
        (insert "# Edit turn entry\n\n")
        (insert "* Notes\n")
        (insert "Write the turn actions here...\n\n")
        (insert "* Consumables\n")
        (let* ((names (mapcar (lambda (j) (plist-get j :name)) players))
               (num-players (length names)))
          (insert "| Item |")
          (dolist (name names)
            (insert (format " %s |" name)))
          (insert "\n")
          ;; Separator
          (insert "|------")
          (dotimes (_ num-players)
            (insert "+------"))
          (insert "|\n"))
        ;; Rows per item (case-insensitive dedup)
        (let* ((items (let ((seen nil)
                            (result nil))
                        (dolist (item (apply #'append
                                             (mapcar (lambda (i) (mapcar #'car (cdr i)))
                                                     inventories)))
                          (let ((key (downcase item)))
                            (unless (member key seen)
                              (push key seen)
                              (push item result))))
                        (nreverse result))))
          (dolist (item items)
            (insert (format "| %s |" item))
            (dolist (inv inventories)
              (let ((quantity (or (cdr (assoc-string item (cdr inv) t)) 0)))
                (insert (format " %.2f |" quantity))))
            (insert "\n")))
        (insert "\n")
        (insert "* Actions\n")
        (insert "- C-c C-c: Save and close\n")
        (insert "- C-c C-k: Cancel\n")
        
        ;; Local variables for saving
        (setq-local dm-log--edit-type type)
        (setq-local dm-log--edit-advance advance)
        (setq-local dm-log--edit-number number)
        (setq-local dm-log--edit-time-start current-time)
        (setq-local dm-log--edit-time-end new-time)
        (setq-local dm-log--edit-players players)
        
        ;; Keymap
        (use-local-map (copy-keymap org-mode-map))
        (local-set-key (kbd "C-c C-c") #'dm-log-transient--save-turn)
        (local-set-key (kbd "C-c C-k") #'dm-log-transient--cancel-turn))
      
      (pop-to-buffer buf)
      (message "Edit notes and consumables. C-c C-c to save."))))

;; -----------------------------------------------------------------------------
;; Save / Cancel turn
;; -----------------------------------------------------------------------------

(defun dm-log-transient--save-turn ()
  "Save the edited turn from the temp buffer."
  (interactive)
  (let* ((type dm-log--edit-type)
         (advance dm-log--edit-advance)
         (number dm-log--edit-number)
         (time-start dm-log--edit-time-start)
         (time-end dm-log--edit-time-end)
         ;; Extract notes
         (notes (save-excursion
                  (goto-char (point-min))
                  (when (re-search-forward "^\\* Notes" nil t)
                    (forward-line)
                    (let ((start (point)))
                      (if (re-search-forward "^\\* " nil t)
                          (buffer-substring-no-properties start (line-beginning-position))
                        (buffer-substring-no-properties start (point-max))))))))

    ;; Extract consumables table
    (let ((consumables-alist nil))
      (save-excursion
        (goto-char (point-min))
        (when (re-search-forward "^\\* Consumables" nil t)
          (forward-line)
          ;; Find table start
          (when (re-search-forward "^\\s-*|" nil t)
            (beginning-of-line)
            (let ((rows (org-table-to-lisp)))
              (when rows
                ;; First row is header with player names
                (let* ((header (car rows))
                       (names (cdr header)))
                  ;; Build alist per player
                  (dolist (name names)
                    (push (cons (string-trim name) nil) consumables-alist))
                  (setq consumables-alist (nreverse consumables-alist))
                  ;; Process item rows (ignore separator)
                  (dolist (row (cdr rows))
                    (when (listp row)
                      (let ((item (string-trim (car row)))
                            (vals (cdr row)))
                        (cl-loop for name-raw in names
                                 for val in vals
                                 for name = (string-trim name-raw)
do (push (cons item (string-to-number (string-trim val)))
                                           (cdr (assoc-string name consumables-alist t)))))))))))))

      ;; Save to org
      (dm-log-org--insert-turn dm-log--current-logbook-file
                                number
                                type
                                advance
                                (string-trim notes)
                                consumables-alist
                                nil
                                time-start
                                time-end)

      ;; Update time
      (dm-log-org--update-current-time dm-log--current-logbook-file time-end)

      ;; Close buffer and refresh
      (kill-buffer (current-buffer))
      (dm-log-campaign--open-logbook)
      (message "Turn %d added." number))))

(defun dm-log-transient--cancel-turn ()
  "Cancel turn editing."
  (interactive)
  (kill-buffer (current-buffer))
  (message "Turn cancelled."))

(provide 'dm-log-transient)
;;; dm-log-transient.el ends here