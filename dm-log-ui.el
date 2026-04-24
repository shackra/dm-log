;;; dm-log-ui.el --- Rendering for *SW-Logbook* buffer  -*- lexical-binding: t; -*-

;;; Commentary:
;; Generates the read-only view of the logbook from the org file.

;;; Code:

(require 'cl-lib)
(require 'org)

(declare-function dm-log-org--get-file-properties "dm-log-org" (file))
(declare-function dm-log--headline-props "dm-log" (hl))

;; -----------------------------------------------------------------------------
;; Buffer-local variables
;; -----------------------------------------------------------------------------

(defvar-local dm-log-ui--campaign-file nil
  "Source org file for the current campaign.")

(defvar-local dm-log-ui--time-format nil
  "Fictitious time format for the campaign.")

;; -----------------------------------------------------------------------------
;; Font Lock
;; -----------------------------------------------------------------------------

(defvar dm-log-ui-font-lock-keywords
  `(;; Section titles
    ("^Logbook of events,.*$" . font-lock-doc-face)
    ;; Turns
    ("^Turn [0-9]+.*$" . font-lock-keyword-face)
    ;; Time jumps
    ("^Time jump:.*$" . font-lock-warning-face)
    ;; Separators
    ("^[_-]\\{40,\\}$" . font-lock-comment-face)
    ;; Labels
    ("^Game time:" . font-lock-type-face)
    ("^Consumables:" . font-lock-type-face))
  "Font-lock keywords for *SW-Logbook*.")

;; -----------------------------------------------------------------------------
;; Rendering
;; -----------------------------------------------------------------------------

(defun dm-log-ui--render (logbook-file)
  "Render LOGBOOK-FILE in the current buffer (*SW-Logbook*)."
  (let* ((props (dm-log-org--get-file-properties logbook-file))
         (fmt (or (plist-get props :time-format) "%B %d, %Y %H:%M"))
         (current-time (plist-get props :current-time))
         (real-time (format-time-string "%B %d, %Y %H:%M"))
         (inhibit-read-only t))
    (erase-buffer)
    (setq dm-log-ui--campaign-file logbook-file)
    (setq dm-log-ui--time-format fmt)

    ;; Header
    (insert "Game time: ")
    (if current-time
        (let ((time-obj (dm-log-time--parse-game-timestamp current-time)))
          (insert (if time-obj
                      (dm-log-time--format-game-timestamp time-obj fmt)
                    current-time)))
      (insert "[Not started]"))
    (insert "\n")
    (insert real-time "\n")
    (insert (make-string 70 ?_))
    (insert "\n\n")

    ;; Content
    (dm-log-ui--render-logbook logbook-file)

    ;; Font lock
    (font-lock-mode -1)
    (font-lock-mode 1)
    (setq font-lock-defaults '(dm-log-ui-font-lock-keywords t))
    (font-lock-ensure)

    ;; Go to start
    (goto-char (point-min))))

(defun dm-log-ui--render-logbook (file)
  "Render the Logbook section of FILE in the current buffer."
  (when (file-exists-p file)
    (let ((parsed (with-temp-buffer
                    (insert-file-contents file)
                    (org-mode)
                    (org-element-parse-buffer)))
          (session-num nil))
      (org-element-map parsed 'headline
        (lambda (hl)
          (let ((level (org-element-property :level hl))
                (title (org-element-property :raw-value hl))
                (props (dm-log--headline-props hl)))
            (cond
             ;; Session (level 2)
             ((= level 2)
              (setq session-num (or (cdr (assoc "NUMBER" props))
                                    (cdr (assoc "NUMERO" props))
                                    "?"))
              (insert (format "Logbook of events, session %s\n\n" session-num)))

             ;; Turn (level 3)
             ((= level 3)
              (let* ((type (or (cdr (assoc "TURN_TYPE" props))
                              (cdr (assoc "TURNO_TIPO" props))))
                     (number (or (cdr (assoc "TURN_NUMBER" props))
                                 (cdr (assoc "TURNO_NUMERO" props))))
                     (advance (or (cdr (assoc "ADVANCE" props))
                                 (cdr (assoc "AVANCE" props))))
                     (is-jump (string= (or (cdr (assoc "ENTRY_TYPE" props))
                                           (cdr (assoc "TIPO_ENTRADA" props))) "time_jump")))
                (if is-jump
                    (insert (format "Time jump: %s (%s)\n" title advance))
                  (insert (format "Turn %s (%s) [%s]\n"
                                  (or number "?")
                                  (or advance "")
                                  (capitalize (or type "?")))))
                ;; Notes
                (let ((notes (dm-log-ui--extract-notes hl)))
                  (when notes
                    (insert notes "\n")))
                ;; Consumables
                (let ((table (dm-log-ui--extract-consumables hl)))
                  (when table
                    (insert "\nConsumables:\n")
                    (dm-log-ui--insert-table table)
                    (insert "\n")))
                (insert "\n"))))))))))

(defun dm-log-ui--extract-notes (headline)
  "Extract text from the Notes sub-heading under HEADLINE."
  (let ((children (org-element-contents headline)))
    (cl-some
     (lambda (child)
       (when (and (eq (org-element-type child) 'headline)
                  (or (string= (org-element-property :raw-value child) "Notes")
                      (string= (org-element-property :raw-value child) "Memo")))
         (let ((text (car (org-element-contents child))))
           (when (stringp text)
             (string-trim text)))))
     children)))

(defun dm-log-ui--extract-consumables (headline)
  "Extract consumables table under HEADLINE.
Return list of lists (table rows)."
  (let ((children (org-element-contents headline)))
    (cl-some
     (lambda (child)
       (when (and (eq (org-element-type child) 'headline)
                  (or (string= (org-element-property :raw-value child) "Consumables")
                      (string= (org-element-property :raw-value child) "Consumibles")))
         (let ((table (cl-find-if
                       (lambda (c) (eq (org-element-type c) 'table))
                       (org-element-contents child))))
           (when table
             (dm-log-ui--parse-org-table table)))))
     children)))

(defun dm-log-ui--parse-org-table (table)
  "Parse org TABLE element to list of rows (lists of cells)."
  (let (rows)
    (org-element-map table 'table-row
      (lambda (row)
        (when (eq (org-element-property :type row) 'standard)
          (let (cells)
            (org-element-map row 'table-cell
              (lambda (cell)
                (let ((content (car (org-element-contents cell))))
                  (push (if (stringp content)
                            (string-trim content)
                          "")
                        cells))))
            (push (nreverse cells) rows)))))
    (nreverse rows)))

(defun dm-log-ui--insert-table (rows)
  "Insert ROWS formatted into the current buffer."
  (when rows
    (let* ((num-cols (length (car rows)))
           (widths (make-list num-cols 0)))
      ;; Calculate widths
      (dolist (row rows)
        (cl-loop for cell in row
                 for i from 0
                 do (setf (nth i widths)
                          (max (nth i widths) (length cell)))))
      ;; Insert
      (dolist (row rows)
        (insert "  ")
        (cl-loop for cell in row
                 for width in widths
                 do (insert (format (format " %%-%ds " width) cell)))
        (insert "\n")))))

(provide 'dm-log-ui)
;;; dm-log-ui.el ends here