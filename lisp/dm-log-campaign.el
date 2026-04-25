;;; dm-log-campaign.el --- Campaign selector and creator  -*- lexical-binding: t; -*-

;;; Commentary:
;; Transient for selecting or creating campaigns from the configured directory.

;;; Code:

(require 'cl-lib)
(require 'transient)
(require 'dm-log-ui)

(declare-function dm-log--campaigns-list "dm-log" ())
(declare-function dm-log--campaign-name "dm-log" (path))
(declare-function dm-log-mode "dm-log" ())

(defvar dm-log-campaigns-directory nil)
(defvar dm-log-logbook-filename nil)
(defvar dm-log-consumables-filename nil)
(defvar dm-log-players-filename nil)
(defvar dm-log-buffer-name nil)
(defvar dm-log--current-campaign nil)
(defvar dm-log--current-logbook-file nil)
(defvar dm-log--current-consumables-file nil)
(defvar dm-log--current-players-file nil)

(declare-function dm-log-campaign-menu "dm-log-campaign" ())

;; -----------------------------------------------------------------------------
;; Campaign selection
;; -----------------------------------------------------------------------------

(defun dm-log-campaign--get-choices ()
  "Return list of campaigns as (name . path) pairs."
  (mapcar (lambda (path)
            (cons (dm-log--campaign-name path) path))
          (dm-log--campaigns-list)))

(defun dm-log-campaign-select ()
  "Show transient to select or create a campaign."
  (interactive)
  (let ((choices (dm-log-campaign--get-choices)))
    ;; Build dynamic transient
    (eval
     `(transient-define-prefix dm-log-campaign-menu ()
        "Campaign Manager"
        ["Actions"
         ("n" "New campaign" dm-log-campaign-create)]
        ["Existing Campaigns"
         ,@(mapcar (lambda (choice)
                     (list (substring (car choice) 0 1)
                           (car choice)
                           (intern (format "dm-log-campaign--load-%s" (car choice)))))
                   choices)]
        (interactive)
        (transient-setup 'dm-log-campaign-menu)))
    ;; Define dynamic load functions
    (dolist (choice choices)
      (let ((name (car choice))
            (path (cdr choice)))
        (eval `(defun ,(intern (format "dm-log-campaign--load-%s" name)) ()
                 ,(format "Load campaign %s" name)
                 (interactive)
                 (dm-log-campaign--load ,path)))))
    ;; Call
    (dm-log-campaign-menu)))

;; -----------------------------------------------------------------------------
;; Campaign creation
;; -----------------------------------------------------------------------------

(defun dm-log-campaign-create (name)
  "Create a new campaign named NAME and open it."
  (interactive "sCampaign name: ")
  (unless (and name (not (string= name "")))
    (user-error "Campaign name cannot be empty"))
  (let* ((dir (expand-file-name name dm-log-campaigns-directory)))
    (when (file-exists-p dir)
      (user-error "Campaign '%s' already exists" name))
    ;; Ensure parent directory exists
    (unless (file-directory-p dm-log-campaigns-directory)
      (make-directory dm-log-campaigns-directory t))
    (make-directory dir)
    ;; Initialize all three files
    (dm-log-campaign--init-logbook
     (expand-file-name dm-log-logbook-filename dir))
    (dm-log-campaign--init-players
     (expand-file-name dm-log-players-filename dir))
    (dm-log-campaign--init-consumables
     (expand-file-name dm-log-consumables-filename dir))
    ;; Load the new campaign
    (dm-log-campaign--load dir)
    (message "Campaign '%s' created and loaded." name)))

;; -----------------------------------------------------------------------------
;; Campaign loading
;; -----------------------------------------------------------------------------

(defun dm-log-campaign--load (path)
  "Load campaign at PATH and open logbook."
  (setq dm-log--current-campaign path)
  (setq dm-log--current-logbook-file
        (expand-file-name dm-log-logbook-filename path))
  (setq dm-log--current-consumables-file
        (expand-file-name dm-log-consumables-filename path))
  (setq dm-log--current-players-file
        (expand-file-name dm-log-players-filename path))

  ;; Verify files exist (scaffold if missing)
  (unless (file-exists-p dm-log--current-logbook-file)
    (dm-log-campaign--init-logbook dm-log--current-logbook-file))
  (unless (file-exists-p dm-log--current-players-file)
    (dm-log-campaign--init-players dm-log--current-players-file))
  (unless (file-exists-p dm-log--current-consumables-file)
    (dm-log-campaign--init-consumables dm-log--current-consumables-file))

  ;; Open logbook
  (dm-log-campaign--open-logbook))

(defun dm-log-campaign--open-logbook ()
  "Open or refresh the *SW-Logbook* buffer with a dired sidebar."
  (let* ((campaign-dir (file-name-directory dm-log--current-logbook-file))
         (dired-buf-name (file-name-nondirectory
                          (directory-file-name campaign-dir)))
         (dired-buf (or (get-buffer dired-buf-name)
                        (dired-noselect campaign-dir)))
         (logbook-buf (get-buffer-create dm-log-buffer-name)))
    (delete-other-windows)
    (split-window-right)
    (other-window 1)
    (switch-to-buffer dired-buf)
    (other-window 1)
    (with-current-buffer logbook-buf
      (setq default-directory campaign-dir)
      (dm-log-mode)
      (dm-log-ui--render dm-log--current-logbook-file))
    (switch-to-buffer logbook-buf)
    (message "Campaign loaded. SPC for menu.")))

;; -----------------------------------------------------------------------------
;; File initialization
;; -----------------------------------------------------------------------------

(defun dm-log-campaign--init-logbook (file)
  "Create initial logbook file."
  (with-temp-file file
    (insert "#+TITLE: New Campaign\n")
    (insert "#+TIME_FORMAT: %B %d, %Y %H:%M\n")
    (insert "#+CURRENT_TIME: [2024-01-01 12:00]\n\n")
    (insert "* Metadata\n")
    (insert "** Players\n")
    (insert "\n* Logbook\n")
    (insert "** Session 1\n")
    (insert ":PROPERTIES:\n")
    (insert ":ID: " (org-id-new) "\n")
    (insert ":NUMBER: 1\n")
    (insert ":REAL_TIME: " (format-time-string "[%Y-%m-%d %a %H:%M]") "\n")
    (insert ":END:\n")))

(defun dm-log-campaign--init-players (file)
  "Create initial players file."
  (with-temp-file file
    (insert "#+TITLE: Players\n\n")
    (insert "* Player A\n")
    (insert ":PROPERTIES:\n")
    (insert ":ID: " (org-id-new) "\n")
    (insert ":TORCHES: 5\n")
    (insert ":RATIONS: 10\n")
    (insert ":OIL: 2\n")
    (insert ":WATER: 3\n")
    (insert ":TORCH-LIT: t\n")
    (insert ":END:\n\n")
    (insert "* Player B\n")
    (insert ":PROPERTIES:\n")
    (insert ":ID: " (org-id-new) "\n")
    (insert ":TORCHES: 3\n")
    (insert ":RATIONS: 8\n")
    (insert ":OIL: 1\n")
    (insert ":WATER: 2\n")
    (insert ":TORCH-LIT: nil\n")
    (insert ":END:\n\n")
    (insert "* Player C\n")
    (insert ":PROPERTIES:\n")
    (insert ":ID: " (org-id-new) "\n")
    (insert ":TORCHES: 4\n")
    (insert ":RATIONS: 12\n")
    (insert ":OIL: 0\n")
    (insert ":WATER: 4\n")
    (insert ":TORCH-LIT: t\n")
    (insert ":END:\n")))

(defun dm-log-campaign--init-consumables (file)
  "Create initial consumables file."
  (with-temp-file file
    (insert "#+TITLE: Consumables Table\n\n")
    (insert "| Item     | Period | Quantity |\n")
    (insert "|----------+--------+----------|\n")
    (insert "| Torches  | 1h     | 1.0      |\n")
    (insert "| Rations  | 24h    | 3.0      |\n")
    (insert "| Oil      | 30m    | 0.5      |\n")
    (insert "| Water    | 8h     | 1.0      |\n")
    (insert "| Arrows   | --     | 0.0      |\n")))

(provide 'dm-log-campaign)
;;; dm-log-campaign.el ends here