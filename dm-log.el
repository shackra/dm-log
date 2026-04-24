;;; dm-log.el --- Sword & Wizardry logbook for Dungeon Masters  -*- lexical-binding: t; -*-

;; Author: Auto-generated
;; Version: 1.0.0
;; Package-Requires: ((emacs "30.1") (transient "0.7.0"))

;;; Commentary:
;; Package for managing Sword & Wizardry campaign logbooks.
;; Provides a read-only *SW-Logbook* buffer with transient navigation.

;;; Code:

(require 'cl-lib)
(require 'org)
(require 'org-element)
(require 'transient)

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

(declare-function dm-log-main-menu "dm-log-transient" ())

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
(declare-function dm-log-campaign-select "dm-log-campaign" ())

(defun dm-log ()
  "Start dm-log showing the campaign selector."
  (interactive)
  (dm-log-campaign-select))

(defun dm-log-quit ()
  "Close the logbook buffer."
  (interactive)
  (when (get-buffer dm-log-buffer-name)
    (kill-buffer dm-log-buffer-name)))

;; -----------------------------------------------------------------------------
;; Utility
;; -----------------------------------------------------------------------------

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