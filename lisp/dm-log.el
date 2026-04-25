;;; dm-log.el --- Sword & Wizardry logbook for Dungeon Masters  -*- lexical-binding: t; -*-

;; Author: Auto-generated
;; Version: 1.0.0
;; Package-Requires: ((emacs "30.1") (transient "0.7.0"))

;;; Commentary:
;; Package for managing Sword & Wizardry campaign logbooks.
;; Provides a read-only *SW-Logbook* buffer with transient navigation.

;;; Code:

(require 'cl-lib)
(require 'dm-log-time)
(require 'dm-log-consumables)
(require 'dm-log-org)
(require 'dm-log-ui)

;; -----------------------------------------------------------------------------
;; Customization
;; -----------------------------------------------------------------------------

(defgroup dm-log nil
  "Sword & Wizardry logbook for Dungeon Masters."
  :group 'games
  :prefix "dm-log-")

(defcustom dm-log-campaigns-directory "~/campaigns"
  "Base directory where campaigns are stored."
  :type 'directory
  :group 'dm-log)

(defcustom dm-log-logbook-filename "logbook.org"
  "Name of the logbook file within the campaign."
  :type 'string
  :group 'dm-log)

(defcustom dm-log-consumables-filename "consumables.org"
  "Name of the consumables rates file within the campaign."
  :type 'string
  :group 'dm-log)

(defcustom dm-log-players-filename "players.org"
  "Name of the players file within the campaign."
  :type 'string
  :group 'dm-log)

(defcustom dm-log-buffer-name "*SW-Logbook*"
  "Name of the main logbook buffer."
  :type 'string
  :group 'dm-log)

(defcustom dm-log-turn-dungeon-advance "10m"
  "Time advance for a dungeon turn."
  :type 'string
  :group 'dm-log)

(defcustom dm-log-turn-wilderness-advance "1h"
  "Time advance for a wilderness turn."
  :type 'string
  :group 'dm-log)

(defcustom dm-log-turn-combat-advance "1m"
  "Time advance for a combat turn."
  :type 'string
  :group 'dm-log)

;; -----------------------------------------------------------------------------
;; State
;; -----------------------------------------------------------------------------

(defvar dm-log--current-campaign nil
  "Absolute path of the active campaign directory.")

(defvar dm-log--current-logbook-file nil
  "Absolute path of the active logbook.org file.")

(defvar dm-log--current-consumables-file nil
  "Absolute path of the active consumables.org file.")

(defvar dm-log--current-players-file nil
  "Absolute path of the active players.org file.")

;; -----------------------------------------------------------------------------
;; Minor Mode
;; -----------------------------------------------------------------------------

(defvar dm-log-mode-map
  (let ((map (make-sparse-keymap)))
    (define-key map (kbd "SPC") #'dm-log-main-menu)
    (define-key map (kbd "q") #'dm-log-quit)
    map)
  "Keymap for `dm-log-mode'.")

(define-minor-mode dm-log-mode
  "Minor mode for the *SW-Logbook* buffer."
  :lighter " SW-Log"
  :keymap dm-log-mode-map
  (setq buffer-read-only t)
  (setq truncate-lines t))

;; -----------------------------------------------------------------------------
;; Entry Point
;; -----------------------------------------------------------------------------

;;;###autoload
(defun dm-log ()
  "Start dm-log showing the campaign selector."
  (interactive)
  (require 'dm-log-campaign)
  (require 'dm-log-transient)
  (dm-log-campaign-select))

(defun dm-log-quit ()
  "Close the logbook buffer, dired sidebar, and any .org buffers from the campaign."
  (interactive)
  (let* ((logbook-buf (get-buffer dm-log-buffer-name))
         (campaign-dir (when dm-log--current-logbook-file
                         (file-name-directory dm-log--current-logbook-file))))
    (when logbook-buf
      (kill-buffer logbook-buf))
    (when campaign-dir
      (dolist (buf (buffer-list))
        (let ((file (buffer-file-name buf)))
          (when (and file
                     (string-prefix-p campaign-dir file))
            (if (buffer-modified-p buf)
                (when (yes-or-no-p (format "Save modified file %s before closing? "
                                            (file-name-nondirectory file)))
                  (with-current-buffer buf (save-buffer))
                  (kill-buffer buf))
              (kill-buffer buf)))))
      (let ((dired-buf (get-buffer (file-name-nondirectory
                                   (directory-file-name campaign-dir)))))
        (when dired-buf
          (kill-buffer dired-buf))))
    (delete-other-windows)))

;; -----------------------------------------------------------------------------
;; Utility
;; -----------------------------------------------------------------------------

(defun dm-log--headline-props (hl)
  "Extract property alist from property-drawer of headline HL.
Uses org-element-map to find node-property elements, which works
across Emacs versions where :properties may be nil."
  (org-element-map hl 'node-property
    (lambda (np)
      (cons (org-element-property :key np)
            (org-element-property :value np)))
    nil nil nil))

(defun dm-log--campaigns-list ()
  "Return list of available campaign directories."
  (when (file-directory-p dm-log-campaigns-directory)
    (cl-remove-if-not
     (lambda (f)
       (and (file-directory-p f)
            (file-exists-p (expand-file-name dm-log-logbook-filename f))))
     (directory-files dm-log-campaigns-directory t "^[^.]"))))

(defun dm-log--campaign-name (path)
  "Extract campaign name from PATH."
  (file-name-nondirectory (directory-file-name path)))

(provide 'dm-log)
;;; dm-log.el ends here