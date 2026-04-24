;;; dm-log-consumables.el --- Consumables calculation  -*- lexical-binding: t; -*-

;;; Commentary:
;; Reads consumables rate table and calculates consumption based on elapsed time.
;; Fixed rate: per unit of time, X quantity is consumed.

;;; Code:

(require 'cl-lib)
(require 'org-table)
(require 'dm-log-time)

;; -----------------------------------------------------------------------------
;; Rate table reading
;; -----------------------------------------------------------------------------

(defun dm-log-consumables--read-table (file)
  "Read FILE (org) and return alist with rates: ((Item . (period . quantity)) ...)."
  (when (file-exists-p file)
    (with-temp-buffer
      (insert-file-contents file)
      (org-mode)
      (goto-char (point-min))
      (let (result)
        (while (re-search-forward "^\\s-*|" nil t)
          (let ((table-start (line-beginning-position)))
            (forward-line)
            (while (and (not (eobp)) (looking-at "^\\s-*|"))
              (forward-line))
            (let ((table-end (point)))
              (save-restriction
                (narrow-to-region table-start table-end)
                (goto-char (point-min))
                (forward-line)
                (when (looking-at "^\\s-*|-")
                  (forward-line))
                (while (not (eobp))
                  (when (looking-at "^\\s-*|\\s-*\\([^|]+\\)|\\s-*\\([^|]+\\)|\\s-*\\([^|]+\\)|")
                    (let ((item (string-trim (match-string 1)))
                          (period (string-trim (match-string 2)))
                          (quantity (string-trim (match-string 3))))
                      (when (and (not (string= item "Item"))
                                 (not (string-prefix-p "-" item)))
                        (push (cons item (cons period (string-to-number quantity))) result))))
                  (forward-line))))))
        (nreverse result)))))

;; -----------------------------------------------------------------------------
;; Consumption calculation
;; -----------------------------------------------------------------------------

(defun dm-log-consumables--calculate-consumption (rates time-period)
  "Calculate consumption for each item in RATES given TIME-PERIOD elapsed.
Return alist: ((Item . quantity-consumed) ...)."
  (let ((time-secs (dm-log-time--parse-period time-period))
        result)
    (when time-secs
      (dolist (rate rates)
        (let* ((item (car rate))
               (period-str (cadr rate))
               (quantity-per-period (cddr rate))
               (period-secs (dm-log-time--parse-period period-str)))
          (when (and period-secs (> period-secs 0))
            (let ((consumption (* (/ (float time-secs) period-secs) quantity-per-period)))
              (push (cons item consumption) result))))))
    (nreverse result)))

(defun dm-log-consumables--apply-consumption (inventory consumption player-props)
  "Apply CONSUMPTION to INVENTORY considering player properties.
INVENTORY: alist ((item . quantity)...)
CONSUMPTION: alist from dm-log-consumables--calculate-consumption
PLAYER-PROPS: player properties (e.g: :TORCH-LIT).
Return new inventory.
Item name matching is case-insensitive so that \"Torches\"
in consumption matches \"TORCHES\" in inventory."
  (let ((new (copy-alist inventory)))
    (dolist (c consumption)
      (let* ((item (car c))
             (qty-consumed (cdr c))
             (current (or (cdr (assoc-string item new t)) 0)))
        (when (and (string= (downcase item) "antorchas")
                   (not (plist-get player-props :TORCH-LIT)))
          (setq qty-consumed 0))
        (let ((inv-key (car (assoc-string item new t))))
          (setf (alist-get (or inv-key item) new nil nil #'string=)
                (max 0 (- current qty-consumed))))))
    new))

(provide 'dm-log-consumables)
;;; dm-log-consumables.el ends here