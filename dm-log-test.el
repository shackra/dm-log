;;; dm-log-test.el --- ERT tests for dm-log  -*- lexical-binding: t; -*-

;;; Code:

(require 'ert)
(require 'cl-lib)
(require 'dm-log-time)
(require 'dm-log-consumables)
(require 'dm-log-org)
(require 'dm-log-ui)
(require 'dm-log)

;; =============================================================================
;; Test helpers
;; =============================================================================

(defmacro dm-log-test-with-temp-file (content &rest body)
  "Create temp file with CONTENT, bind `file', execute BODY, cleanup."
  (declare (indent 1) (debug t))
  `(let* ((file (make-temp-file "dm-log-test-" nil ".org")))
     (unwind-protect
         (progn
           (with-temp-buffer
             (insert ,content)
             (write-region (point-min) (point-max) file))
           ,@body)
       (when (file-exists-p file)
         (delete-file file)))))

(defmacro dm-log-test-with-parsed-org (content var &rest body)
  "Insert CONTENT in temp buffer, parse with org-element, bind result to VAR."
  (declare (indent 2) (debug t))
  `(with-temp-buffer
     (insert ,content)
     (org-mode)
     (let ((,var (org-element-parse-buffer)))
       ,@body)))

;; =============================================================================
;; Tier 1: Pure time functions
;; =============================================================================

;; --- dm-log-time--parse-period ---

(ert-deftest dm-log-test-parse-period-minutes ()
  (should (= (dm-log-time--parse-period "10m") 600)))

(ert-deftest dm-log-test-parse-period-hours ()
  (should (= (dm-log-time--parse-period "2h") 7200)))

(ert-deftest dm-log-test-parse-period-days ()
  (should (= (dm-log-time--parse-period "1d") 86400)))

(ert-deftest dm-log-test-parse-period-weeks ()
  (should (= (dm-log-time--parse-period "2w") 1209600)))

(ert-deftest dm-log-test-parse-period-seconds ()
  (should (= (dm-log-time--parse-period "30s") 30)))

(ert-deftest dm-log-test-parse-period-combat ()
  (should (= (dm-log-time--parse-period "1m") 60)))

(ert-deftest dm-log-test-parse-period-invalid-letters ()
  (should-not (dm-log-time--parse-period "5x"))
  (should-not (dm-log-time--parse-period "abc"))
  (should-not (dm-log-time--parse-period "")))

(ert-deftest dm-log-test-parse-period-no-number ()
  (should-not (dm-log-time--parse-period "m"))
  (should-not (dm-log-time--parse-period "h")))

(ert-deftest dm-log-test-parse-period-zero ()
  (should (= (dm-log-time--parse-period "0m") 0)))

;; --- dm-log-time--period-to-string ---

(ert-deftest dm-log-test-period-to-string-weeks ()
  (should (string= (dm-log-time--period-to-string 604800) "1w"))
  (should (string= (dm-log-time--period-to-string 1209600) "2w")))

(ert-deftest dm-log-test-period-to-string-days ()
  (should (string= (dm-log-time--period-to-string 86400) "1d"))
  (should (string= (dm-log-time--period-to-string 172800) "2d")))

(ert-deftest dm-log-test-period-to-string-hours ()
  (should (string= (dm-log-time--period-to-string 3600) "1h"))
  (should (string= (dm-log-time--period-to-string 7200) "2h")))

(ert-deftest dm-log-test-period-to-string-minutes ()
  (should (string= (dm-log-time--period-to-string 60) "1m"))
  (should (string= (dm-log-time--period-to-string 600) "10m")))

(ert-deftest dm-log-test-period-to-string-seconds ()
  (should (string= (dm-log-time--period-to-string 30) "30s")))

(ert-deftest dm-log-test-period-roundtrip ()
  (dolist (input '("10m" "2h" "1d" "1w" "5s" "30s"))
    (let ((secs (dm-log-time--parse-period input)))
      (should secs)
      (should (string= (dm-log-time--period-to-string secs) input)))))

;; --- dm-log-time--parse-game-timestamp ---

(ert-deftest dm-log-test-parse-timestamp-valid ()
  (let ((time (dm-log-time--parse-game-timestamp "[2024-01-15 08:30]")))
    (should time)
    (let ((decoded (decode-time time)))
      (should (= (decoded-time-year decoded) 2024))
      (should (= (decoded-time-month decoded) 1))
      (should (= (decoded-time-day decoded) 15))
      (should (= (decoded-time-hour decoded) 8))
      (should (= (decoded-time-minute decoded) 30)))))

(ert-deftest dm-log-test-parse-timestamp-invalid ()
  (should-not (dm-log-time--parse-game-timestamp "not-a-timestamp"))
  (should-not (dm-log-time--parse-game-timestamp ""))
  (should-not (dm-log-time--parse-game-timestamp "[2024-1-15 08:30]")))

;; --- dm-log-time--format-game-timestamp ---

(ert-deftest dm-log-test-format-timestamp-default ()
  (let ((time (dm-log-time--parse-game-timestamp "[2024-03-05 14:22]")))
    (should (string= (dm-log-time--format-game-timestamp time)
                     "[2024-03-05 14:22]"))))

(ert-deftest dm-log-test-format-timestamp-custom-fmt ()
  (let ((time (dm-log-time--parse-game-timestamp "[2024-03-05 14:22]")))
    (should (string= (dm-log-time--format-game-timestamp time "%Y-%m-%d")
                     "2024-03-05"))))

(ert-deftest dm-log-test-timestamp-roundtrip ()
  (let ((original "[2024-12-31 23:59]"))
    (should (string= (dm-log-time--format-game-timestamp
                      (dm-log-time--parse-game-timestamp original))
                     original))))

;; --- dm-log-time--add-period ---

(ert-deftest dm-log-test-add-period-ten-minutes ()
  (let* ((start (dm-log-time--parse-game-timestamp "[2024-01-01 00:00]"))
         (result (dm-log-time--add-period start "10m")))
    (should result)
    (should (string= (dm-log-time--format-game-timestamp result)
                     "[2024-01-01 00:10]"))))

(ert-deftest dm-log-test-add-period-one-hour ()
  (let* ((start (dm-log-time--parse-game-timestamp "[2024-01-01 12:00]"))
         (result (dm-log-time--add-period start "1h")))
    (should result)
    (should (string= (dm-log-time--format-game-timestamp result)
                     "[2024-01-01 13:00]"))))

(ert-deftest dm-log-test-add-period-day-rollover ()
  (let* ((start (dm-log-time--parse-game-timestamp "[2024-01-01 23:30]"))
         (result (dm-log-time--add-period start "1h")))
    (should result)
    (should (string= (dm-log-time--format-game-timestamp result)
                     "[2024-01-02 00:30]"))))

(ert-deftest dm-log-test-add-period-invalid ()
  (should-not (dm-log-time--add-period
               (dm-log-time--parse-game-timestamp "[2024-01-01 00:00]")
               "5x")))

;; =============================================================================
;; Tier 2: Org-element helpers
;; =============================================================================

;; --- dm-log--headline-props ---

(ert-deftest dm-log-test-headline-props-with-drawer ()
  (dm-log-test-with-parsed-org
      "* Test
:PROPERTIES:
:TURN_NUMBER: 5
:TURN_TYPE: dungeon
:END:
"   tree
    (let ((hl (org-element-map tree 'headline #'identity nil t)))
      (should hl)
      (let ((props (dm-log--headline-props hl)))
        (should props)
        (should (string= (cdr (assoc "TURN_NUMBER" props)) "5"))
        (should (string= (cdr (assoc "TURN_TYPE" props)) "dungeon"))))))

(ert-deftest dm-log-test-headline-props-no-drawer ()
  (dm-log-test-with-parsed-org
      "* NoProps
Some text
"   tree
    (let ((hl (org-element-map tree 'headline #'identity nil t)))
      (should hl)
      (should-not (dm-log--headline-props hl)))))

;; --- dm-log-ui--extract-notes ---

(ert-deftest dm-log-test-extract-notes-english ()
  (dm-log-test-with-parsed-org
      "*** Turn 1 [dungeon] (10m)
:PROPERTIES:
:TURN_NUMBER: 1
:END:
**** Notes
The party entered the dungeon.
"   tree
    (let* ((hls (org-element-map tree 'headline #'identity))
           (turn (cl-find-if
                  (lambda (h) (string= (org-element-property :raw-value h) "Turn 1 [dungeon] (10m)"))
                  hls)))
      (should turn)
      (let ((notes (dm-log-ui--extract-notes turn)))
        (should notes)
        (should (string= notes "The party entered the dungeon."))))))

(ert-deftest dm-log-test-extract-notes-memo ()
  (dm-log-test-with-parsed-org
      "*** Turn 1 [dungeon] (10m)
:PROPERTIES:
:TURN_NUMBER: 1
:END:
**** Memo
Los personajes entraron.
"   tree
    (let* ((hls (org-element-map tree 'headline #'identity))
           (turn (cl-find-if
                  (lambda (h) (string= (org-element-property :raw-value h) "Turn 1 [dungeon] (10m)"))
                  hls)))
      (should turn)
      (let ((notes (dm-log-ui--extract-notes turn)))
        (should notes)
        (should (string= notes "Los personajes entraron."))))))

(ert-deftest dm-log-test-extract-notes-missing ()
  (dm-log-test-with-parsed-org
      "*** Turn 1 [dungeon] (10m)
:PROPERTIES:
:TURN_NUMBER: 1
:END:
"   tree
    (let* ((hls (org-element-map tree 'headline #'identity))
           (turn (cl-find-if
                  (lambda (h) (string= (org-element-property :raw-value h) "Turn 1 [dungeon] (10m)"))
                  hls)))
      (should turn)
      (should-not (dm-log-ui--extract-notes turn)))))

;; --- dm-log-ui--extract-consumables ---

(ert-deftest dm-log-test-extract-consumables-english ()
  (dm-log-test-with-parsed-org
      "*** Turn 1 [dungeon] (10m)
:PROPERTIES:
:TURN_NUMBER: 1
:END:
**** Consumables
| Item      | Aragorn | Boromir |
|-----------+---------+---------|
| Torches   | 5.00    | 3.00    |
| Rations   | 2.00    | 1.00    |
"   tree
    (let* ((hls (org-element-map tree 'headline #'identity))
           (turn (cl-find-if
                  (lambda (h) (string= (org-element-property :raw-value h) "Turn 1 [dungeon] (10m)"))
                  hls)))
      (should turn)
      (let ((table (dm-log-ui--extract-consumables turn)))
        (should table)
        (should (> (length table) 0))))))

(ert-deftest dm-log-test-extract-consumables-spanish ()
  (dm-log-test-with-parsed-org
      "*** Turno 1 [dungeon] (10m)
:PROPERTIES:
:TURNO_NUMERO: 1
:END:
**** Consumibles
| Item    | Aragorn |
|---------+---------|
| Antorchas | 5.00    |
"   tree
    (let* ((hls (org-element-map tree 'headline #'identity))
           (turn (cl-find-if
                  (lambda (h) (string= (org-element-property :raw-value h) "Turno 1 [dungeon] (10m)"))
                  hls)))
      (should turn)
      (let ((table (dm-log-ui--extract-consumables turn)))
        (should table)))))

(ert-deftest dm-log-test-extract-consumables-missing ()
  (dm-log-test-with-parsed-org
      "*** Turn 1 [dungeon] (10m)
:PROPERTIES:
:TURN_NUMBER: 1
:END:
**** Notes
Just notes here.
"   tree
    (let* ((hls (org-element-map tree 'headline #'identity))
           (turn (cl-find-if
                  (lambda (h) (string= (org-element-property :raw-value h) "Turn 1 [dungeon] (10m)"))
                  hls)))
      (should turn)
      (should-not (dm-log-ui--extract-consumables turn)))))

;; --- dm-log-ui--parse-org-table ---

(ert-deftest dm-log-test-parse-org-table-simple ()
  (dm-log-test-with-parsed-org
      "| Item    | Qty |
|---------+-----|
| Torches | 5   |
"   tree
    (let ((table (org-element-map tree 'table #'identity nil t)))
      (should table)
      (let ((rows (dm-log-ui--parse-org-table table)))
        (should rows)
        (should (= (length rows) 2))
        (should (equal (car rows) '("Item" "Qty")))
        (should (equal (cadr rows) '("Torches" "5")))))))

(ert-deftest dm-log-test-parse-org-table-skips-hline ()
  (dm-log-test-with-parsed-org
      "| Item    | Qty |
|---------+-----|
| Torches | 5   |
| Rations | 3   |
"   tree
    (let ((table (org-element-map tree 'table #'identity nil t)))
      (should table)
      (let ((rows (dm-log-ui--parse-org-table table)))
        (should (= (length rows) 3))
        (should (equal (car rows) '("Item" "Qty")))
        (should (equal (cadr rows) '("Torches" "5")))
        (should (equal (caddr rows) '("Rations" "3")))))))

(ert-deftest dm-log-test-parse-org-table-empty-cells ()
  (dm-log-test-with-parsed-org
      "| A | B |
|---+---|
| x |   |
"   tree
    (let ((table (org-element-map tree 'table #'identity nil t)))
      (should table)
      (let ((rows (dm-log-ui--parse-org-table table)))
        (should (= (length rows) 2))
        (should (equal (cadr rows) '("x" "")))))))

;; --- dm-log-ui--insert-table ---

(ert-deftest dm-log-test-insert-table-simple ()
  (with-temp-buffer
    (dm-log-ui--insert-table '(("Item" "Aragorn") ("Torches" "5") ("Rations" "3")))
    (let ((text (buffer-string)))
      (should (string-match "Item" text))
      (should (string-match "Aragorn" text))
      (should (string-match "Torches" text))
      (should (string-match "5" text)))))

(ert-deftest dm-log-test-insert-table-empty ()
  (with-temp-buffer
    (dm-log-ui--insert-table nil)
    (should (string= (buffer-string) ""))))

(ert-deftest dm-log-test-insert-table-column-alignment ()
  (with-temp-buffer
    (dm-log-ui--insert-table '(("Item" "Player1") ("Torches" "5")))
    (let ((lines (split-string (buffer-string) "\n" t)))
      (should (= (length lines) 2))
      (should (string-match "Item\\s-+Player1" (car lines)))
      (should (string-match "Torches\\s-+5" (cadr lines))))))

;; =============================================================================
;; Tier 3: File-based reader tests
;; =============================================================================

;; --- dm-log-org--get-file-properties ---

(ert-deftest dm-log-test-get-file-properties-keywords ()
  (dm-log-test-with-temp-file
      "#+TIME_FORMAT: %B %d, %Y %H:%M
#+CURRENT_TIME: [2024-01-15 08:30]
* Logbook
"
    (let ((props (dm-log-org--get-file-properties file)))
      (should props)
      (should (string= (plist-get props :time-format) "%B %d, %Y %H:%M"))
      (should (string= (plist-get props :current-time) "[2024-01-15 08:30]")))))

(ert-deftest dm-log-test-get-file-properties-spanish-keywords ()
  (dm-log-test-with-temp-file
      "#+FORMATO_TIEMPO: %d/%m/%Y %H:%M
#+TIEMPO_ACTUAL: [2024-03-01 12:00]
* Logbook
"
    (let ((props (dm-log-org--get-file-properties file)))
      (should props)
      (should (string= (plist-get props :time-format) "%d/%m/%Y %H:%M"))
      (should (string= (plist-get props :current-time) "[2024-03-01 12:00]")))))

(ert-deftest dm-log-test-get-file-properties-missing-file ()
  (should-not (dm-log-org--get-file-properties "/nonexistent/file.org")))

;; --- dm-log-org--get-players ---

(ert-deftest dm-log-test-get-players-basic ()
  (dm-log-test-with-temp-file
      "* Aragorn
:PROPERTIES:
:ID: uuid-1
:TORCH-LIT: t
:TORCHES: 5
:RATIONS: 3
:END:

* Boromir
:PROPERTIES:
:ID: uuid-2
:TORCHES: 3
:RATIONS: 1
:END:
"
    (let ((players (dm-log-org--get-players file)))
      (should players)
      (should (= (length players) 2))
      (let ((aragorn (cl-find "Aragorn" players :key (lambda (p) (plist-get p :name)) :test #'string=)))
        (should aragorn)
        (should (plist-get (plist-get aragorn :props) :torch-lit))
        (let ((inv (plist-get aragorn :inventory)))
          (should inv)
          (should (assoc "TORCHES" inv))
          (should (assoc "RATIONS" inv))))
      (let ((boromir (cl-find "Boromir" players :key (lambda (p) (plist-get p :name)) :test #'string=)))
        (should boromir)
        (should-not (plist-get (plist-get boromir :props) :torch-lit))))))

(ert-deftest dm-log-test-get-players-spanish ()
  (dm-log-test-with-temp-file
      "* Aragorn
:PROPERTIES:
:ID: uuid-1
:ANTORCHA_ENCENDIDA: t
:ANTORCHAS: 5
:RACIONES: 3
:END:
"
    (let ((players (dm-log-org--get-players file)))
      (should players)
      (should (= (length players) 1))
      (let ((p (car players)))
        (should (string= (plist-get p :name) "Aragorn"))
        (should (plist-get (plist-get p :props) :torch-lit))))))

(ert-deftest dm-log-test-get-players-missing-file ()
  (should-not (dm-log-org--get-players "/nonexistent/players.org")))

;; --- dm-log-org--get-last-entry ---

(ert-deftest dm-log-test-get-last-entry-basic ()
  (dm-log-test-with-temp-file
      "* Logbook
:PROPERTIES:
:CURRENT_TIME: [2024-01-15 08:40]
:END:

** Session 1
:PROPERTIES:
:NUMBER: 1
:END:

*** Turn 1 [dungeon] (10m)
:PROPERTIES:
:TURN_NUMBER: 1
:TURN_TYPE: dungeon
:TIME_START: [2024-01-15 08:30]
:TIME_END: [2024-01-15 08:40]
:ADVANCE: 10m
:END:
**** Notes
The party entered the dungeon.
"
    (let ((entry (dm-log-org--get-last-entry file)))
      (should entry)
      (should (= (plist-get entry :number) 1))
      (should (string= (plist-get entry :type) "dungeon"))
      (should (string= (plist-get entry :advance) "10m")))))

(ert-deftest dm-log-test-get-last-entry-spanish ()
  (dm-log-test-with-temp-file
      "* Logbook
:PROPERTIES:
:TIEMPO_ACTUAL: [2024-01-15 08:40]
:END:

** Session 1
:PROPERTIES:
:NUMERO: 1
:END:

*** Turno 1 [dungeon] (10m)
:PROPERTIES:
:TURNO_NUMERO: 1
:TURNO_TIPO: dungeon
:TIEMPO_INICIO: [2024-01-15 08:30]
:TIEMPO_FIN: [2024-01-15 08:40]
:AVANCE: 10m
:END:
**** Notas
Los personajes entraron.
"
    (let ((entry (dm-log-org--get-last-entry file)))
      (should entry)
      (should (= (plist-get entry :number) 1))
      (should (string= (plist-get entry :type) "dungeon"))
      (should (string= (plist-get entry :advance) "10m")))))

(ert-deftest dm-log-test-get-last-entry-no-turns ()
  (dm-log-test-with-temp-file
      "* Logbook
:PROPERTIES:
:CURRENT_TIME: [2024-01-15 08:30]
:END:

** Session 1
:PROPERTIES:
:NUMBER: 1
:END:
"
    (should-not (dm-log-org--get-last-entry file))))

(ert-deftest dm-log-test-get-last-entry-missing-file ()
  (should-not (dm-log-org--get-last-entry "/nonexistent/logbook.org")))

;; --- dm-log-consumables--read-table ---

(ert-deftest dm-log-test-consumables-read-table ()
  (dm-log-test-with-temp-file
      "| Item      | Period | Quantity |
|-----------+--------+----------|
| Torches   | 1h     | 1.0      |
| Rations   | 24h    | 3.0      |
"
    (let ((table (dm-log-consumables--read-table file)))
      (should table)
      (should (= (length table) 2))
      (let ((torches (assoc "Torches" table)))
        (should torches)
        (should (string= (cadr torches) "1h"))
        (should (= (cddr torches) 1.0)))
      (let ((rations (assoc "Rations" table)))
        (should rations)
        (should (string= (cadr rations) "24h"))
        (should (= (cddr rations) 3.0))))))

(ert-deftest dm-log-test-consumables-read-table-missing-file ()
  (should-not (dm-log-consumables--read-table "/nonexistent/consumables.org")))

;; --- dm-log-consumables--calculate-consumption ---

(ert-deftest dm-log-test-calculate-consumption-basic ()
  (let* ((rates '(("Torches" . ("1h" . 1.0))
                  ("Rations" . ("24h" . 3.0))))
         (result (dm-log-consumables--calculate-consumption rates "1h")))
    (should result)
    (let ((torches (assoc "Torches" result)))
      (should torches)
      (should (= (cdr torches) 1.0)))
    (let ((rations (assoc "Rations" result)))
      (should rations)
      (should (< (abs (- (cdr rations) 0.125)) 0.01)))))

(ert-deftest dm-log-test-calculate-consumption-zero-time ()
  (let* ((rates '(("Torches" . ("1h" . 1.0))))
         (result (dm-log-consumables--calculate-consumption rates "0s")))
    (should result)
    (should (= (cdr (car result)) 0.0))))

(ert-deftest dm-log-test-calculate-consumption-invalid-time ()
  (let* ((rates '(("Torches" . ("1h" . 1.0)))))
    (should-not (dm-log-consumables--calculate-consumption rates "invalid"))))

;; --- dm-log-consumables--apply-consumption ---

(ert-deftest dm-log-test-apply-consumption-basic ()
  (let* ((inventory '(("Torches" . 10) ("Rations" . 6)))
         (consumption '(("Torches" . 2.0) ("Rations" . 0.5)))
         (result (dm-log-consumables--apply-consumption inventory consumption nil)))
    (should result)
    (should (= (cdr (assoc "Torches" result)) 8.0))
    (should (= (cdr (assoc "Rations" result)) 5.5))))

(ert-deftest dm-log-test-apply-consumption-torch-lit ()
  (let* ((inventory '(("Antorchas" . 10)))
         (consumption '(("Antorchas" . 2.0)))
         (result (dm-log-consumables--apply-consumption inventory consumption '(:TORCH-LIT t))))
    (should result)
    (should (= (cdr (assoc "Antorchas" result)) 8.0))))

(ert-deftest dm-log-test-apply-consumption-torch-not-lit ()
  (let* ((inventory '(("Antorchas" . 10)))
         (consumption '(("Antorchas" . 2.0)))
         (result (dm-log-consumables--apply-consumption inventory consumption nil)))
    (should result)
    (should (= (cdr (assoc "Antorchas" result)) 10.0))))

(ert-deftest dm-log-test-apply-consumption-no-negative ()
  (let* ((inventory '(("Torches" . 1)))
         (consumption '(("Torches" . 5.0)))
         (result (dm-log-consumables--apply-consumption inventory consumption nil)))
    (should result)
    (should (= (cdr (assoc "Torches" result)) 0.0))))

;; =============================================================================
;; Tier 4: Integration — render cycle
;; =============================================================================

(defconst dm-log-test-logbook-fixture
  "#+TITLE: Test Campaign
#+TIME_FORMAT: %B %d, %Y %H:%M
#+CURRENT_TIME: [2024-01-15 08:40]

* Logbook
:PROPERTIES:
:CURRENT_TIME: [2024-01-15 08:40]
:END:

** Session 1
:PROPERTIES:
:NUMBER: 1
:REAL_TIME: [2024-06-01]
:END:

*** Turn 1 [dungeon] (10m)
:PROPERTIES:
:TURN_NUMBER: 1
:TURN_TYPE: dungeon
:TIME_START: [2024-01-15 08:30]
:TIME_END: [2024-01-15 08:40]
:ADVANCE: 10m
:END:
**** Notes
The party enters the dungeon.
**** Consumables
| Item    | Aragorn | Boromir |
|---------+---------+---------|
| Torches | 5.00    | 3.00    |
| Rations | 2.00    | 1.00    |

*** Time jump: Rest period (8h)
:PROPERTIES:
:ENTRY_TYPE: time_jump
:TIME_START: [2024-01-15 08:40]
:TIME_END: [2024-01-15 16:40]
:ADVANCE: 8h
:END:
**** Notes
The party slept.
"
  "Standard logbook fixture for integration tests.")

(ert-deftest dm-log-test-render-logbook-turns-and-jumps ()
  (dm-log-test-with-temp-file dm-log-test-logbook-fixture
    (with-temp-buffer
      (dm-log-ui--render-logbook file)
      (let ((text (buffer-string)))
        (should (string-match "Logbook of events, session 1" text))
        (should (string-match "Turn 1 (10m) \\[Dungeon\\]" text))
        (should (string-match "Time jump: Rest period (8h)" text))
        (should (string-match "The party enters the dungeon" text))
        (should (string-match "The party slept" text))))))

(ert-deftest dm-log-test-render-logbook-consumables ()
  (dm-log-test-with-temp-file dm-log-test-logbook-fixture
    (with-temp-buffer
      (dm-log-ui--render-logbook file)
      (let ((text (buffer-string)))
        (should (string-match "Consumables:" text))
        (should (string-match "Torches" text))
        (should (string-match "Aragorn" text))))))

(ert-deftest dm-log-test-render-full-buffer ()
  (dm-log-test-with-temp-file dm-log-test-logbook-fixture
    (with-temp-buffer
      (dm-log-ui--render file)
      (let ((text (buffer-string)))
        (should (string-match "^Game time:" text))
        (should (string-match "Logbook of events, session 1" text))
        (should (string-match "^___" text))))))

(ert-deftest dm-log-test-render-missing-file ()
  (with-temp-buffer
    (dm-log-ui--render "/nonexistent/logbook.org")
    (let ((text (buffer-string)))
      (should (string-match "Game time:" text))
      (should (string-match "\\[Not started\\]" text)))))

;; =============================================================================
;; Tier 5: File write tests
;; =============================================================================

;; --- dm-log-org--insert-turn ---

(ert-deftest dm-log-test-insert-turn-basic ()
  (dm-log-test-with-temp-file
      "* Logbook\n** Session 1\n:PROPERTIES:\n:NUMBER: 1\n:END:\n"
    (let* ((time-start (dm-log-time--parse-game-timestamp "[2024-01-15 08:30]"))
           (time-end (dm-log-time--parse-game-timestamp "[2024-01-15 08:40]"))
           (consumables '(("Aragorn" . (("Torches" . 5.0) ("Rations" . 2.0)))
                          ("Boromir" . (("Torches" . 3.0) ("Rations" . 1.0))))))
      (dm-log-org--insert-turn file 1 "dungeon" "10m" "The party enters." consumables nil time-start time-end)
      (should (file-exists-p file))
      (let ((entry (dm-log-org--get-last-entry file)))
        (should entry)
        (should (= (plist-get entry :number) 1))
        (should (string= (plist-get entry :type) "dungeon"))
        (should (string= (plist-get entry :advance) "10m"))))))

;; --- dm-log-org--insert-time-jump ---

(ert-deftest dm-log-test-insert-time-jump-basic ()
  (dm-log-test-with-temp-file
      "* Logbook\n** Session 1\n:PROPERTIES:\n:NUMBER: 1\n:END:\n"
    (let* ((time-start (dm-log-time--parse-game-timestamp "[2024-01-15 08:40]"))
           (time-end (dm-log-time--parse-game-timestamp "[2024-01-15 16:40]")))
      (dm-log-org--insert-time-jump file "Rest period" time-start time-end "8h")
      (should (file-exists-p file))
      (let ((content (with-temp-buffer
                        (insert-file-contents file)
                        (buffer-string))))
        (should (string-match "Time jump: Rest period" content))
        (should (string-match ":ENTRY_TYPE: time_jump" content))
        (should (string-match ":ADVANCE: 8h" content))))))

;; --- dm-log-org--update-current-time ---

(ert-deftest dm-log-test-update-current-time-keyword ()
  (dm-log-test-with-temp-file
      "#+TITLE: Test\n#+CURRENT_TIME: [2024-01-15 08:30]\n* Logbook\n"
    (let ((new-time (dm-log-time--parse-game-timestamp "[2024-01-15 08:40]")))
      (dm-log-org--update-current-time file new-time)
      (with-temp-buffer
        (insert-file-contents file)
        (should (re-search-forward "#\\+CURRENT_TIME:" nil t))
        (should (re-search-forward "08:40" nil t))))))

(ert-deftest dm-log-test-update-current-time-property-drawer ()
  (dm-log-test-with-temp-file
      "* Logbook
:PROPERTIES:
:CURRENT_TIME: [2024-01-15 08:30]
:END:
"
    (let ((new-time (dm-log-time--parse-game-timestamp "[2024-01-15 08:40]")))
      (dm-log-org--update-current-time file new-time)
      (with-temp-buffer
        (insert-file-contents file)
        (should (re-search-forward "CURRENT_TIME" nil t))
        (should (re-search-forward "08:40" nil t))))))

;; =============================================================================
;; Extra edge-case tests
;; =============================================================================

(ert-deftest dm-log-test-render-logbook-spanish-keys ()
  (dm-log-test-with-temp-file
      "#+TITLE: Test
#+CURRENT_TIME: [2024-01-15 08:40]

* Logbook
:PROPERTIES:
:TIEMPO_ACTUAL: [2024-01-15 08:40]
:END:

** Session 1
:PROPERTIES:
:NUMERO: 1
:END:

*** Turno 1 [dungeon] (10m)
:PROPERTIES:
:TURNO_NUMERO: 1
:TURNO_TIPO: dungeon
:AVANCE: 10m
:TIEMPO_INICIO: [2024-01-15 08:30]
:TIEMPO_FIN: [2024-01-15 08:40]
:END:
**** Memo
Los personajes entraron.
"
    (with-temp-buffer
      (dm-log-ui--render-logbook file)
      (let ((text (buffer-string)))
        (should (string-match "Logbook of events, session 1" text))
        (should (string-match "Turn 1 (10m) \\[Dungeon\\]" text))
        (should (string-match "Los personajes entraron" text))))))

(ert-deftest dm-log-test-render-logbook-missing-entry-type ()
  (dm-log-test-with-temp-file
      "#+CURRENT_TIME: [2024-01-15 08:40]

* Logbook
** Session 1
:PROPERTIES:
:NUMBER: 1
:END:

*** Turn 1
:PROPERTIES:
:TURN_NUMBER: 1
:TURN_TYPE: wilderness
:ADVANCE: 1h
:END:
"
    (with-temp-buffer
      (dm-log-ui--render-logbook file)
      (let ((text (buffer-string)))
        (should (string-match "Turn 1 (1h) \\[Wilderness\\]" text))))))

(ert-deftest dm-log-test-campaigns-list-nonexistent-dir ()
  (let ((dm-log-campaigns-directory "/nonexistent/dir/12345"))
    (should-not (dm-log--campaigns-list))))

(provide 'dm-log-test)
;;; dm-log-test.el ends here