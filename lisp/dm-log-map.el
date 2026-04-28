;;; dm-log-map.el --- ASCII map editor integration for dm-log  -*- lexical-binding: t; -*-

;;; Commentary:
;; Launches the mazaforja TUI map editor in an `eat' terminal buffer.
;; Also provides `dm-log-map--open-keying-buffer', called by the Rust
;; process via emacsclient when the DM keys a location.

;;; Code:

(require 'cl-lib)

(declare-function eat "eat" (program &optional arg))

;; ---------------------------------------------------------------------------
;; Customization
;; ---------------------------------------------------------------------------

(defcustom dm-log-map-binary "mazaforja"
  "Path or name of the mazaforja map editor binary."
  :type 'string
  :group 'dm-log)

(defcustom dm-log-map-buffer-name "*map-editor*"
  "Name of the eat buffer used for the map editor."
  :type 'string
  :group 'dm-log)

;; ---------------------------------------------------------------------------
;; Launch
;; ---------------------------------------------------------------------------

;;;###autoload
(defun dm-log-map ()
  "Open the mazaforja ASCII map editor for the current campaign.
Requires `dm-log--current-campaign' to be set (i.e. a campaign must be
loaded via `dm-log')."
  (interactive)
  (unless (bound-and-true-p dm-log--current-campaign)
    (user-error "No campaign loaded.  Run M-x dm-log first"))
  (unless (require 'eat nil t)
    (user-error "Package `eat' is not installed.  Install it with M-x package-install RET eat"))
  (let* ((campaign-dir dm-log--current-campaign)
         (cmd (format "%s --campaign-dir %s"
                      dm-log-map-binary
                      (shell-quote-argument (expand-file-name campaign-dir)))))
    (if-let ((buf (get-buffer dm-log-map-buffer-name)))
        (pop-to-buffer buf)
      (eat cmd)
      (when-let ((buf (get-buffer dm-log-map-buffer-name)))
        (with-current-buffer buf
          (rename-buffer dm-log-map-buffer-name))))))

;; ---------------------------------------------------------------------------
;; Keying buffer (called from Rust via emacsclient)
;; ---------------------------------------------------------------------------

;;;###autoload
(defun dm-log-map--open-keying-buffer (uuid entity-type campaign-dir)
  "Open map.org in CAMPAIGN-DIR and jump to (or create) heading with :ID: UUID.
ENTITY-TYPE is a string like \"hex\", \"room\", \"zone\" — used as the heading
tag when creating a new entry.
This function is meant to be called synchronously by the mazaforja process
via `emacsclient --eval'."
  (require 'org-id)
  (let* ((map-org (expand-file-name "map.org" campaign-dir))
         (buf (find-file-noselect map-org)))
    (with-current-buffer buf
      (unless (derived-mode-p 'org-mode)
        (org-mode))
      ;; Try to find existing heading with this UUID
      (let ((marker (org-id-find uuid t)))
        (if marker
            ;; Jump to existing heading
            (progn
              (pop-to-buffer (marker-buffer marker))
              (goto-char (marker-position marker))
              (org-show-entry))
          ;; Create new heading at end of file
          (goto-char (point-max))
          (unless (bolp) (insert "\n"))
          (insert (format "* %s :%s:\n:PROPERTIES:\n:ID: %s\n:END:\n\n"
                          (capitalize entity-type) entity-type uuid))
          (pop-to-buffer buf)
          (goto-char (point-max))
          (forward-line -2))))))

;; ---------------------------------------------------------------------------

(provide 'dm-log-map)
;;; dm-log-map.el ends here
