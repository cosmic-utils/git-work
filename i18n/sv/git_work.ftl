app-title = GitHub aviseringar
about = Om
notifications = Aviseringar
loading = Laddar aviseringar...
error = Fel
no-notifications = 🎉 Allt ikapp!
no-notifications-desc = Inga nya new aviseringar
show-all = Visa alla
mark-all-read = Markera alla lästa
mark-as-read = Markera som läst
refresh = Uppdatera
notifications-count = { $count ->
    [one] { $count } avisering
    *[other] { $count } aviseringar
}

# Felmeddelanden
error-no-token = GitHub-token hittades inte. Vänligen ange miljövariabeln GITHUB_TOKEN.
error-token-setup = För att åtgärda detta:
error-token-step1 = 1. Skapa en personlig åtkomsttoken på GitHub
error-token-step2 = 2. Ställ in miljövariabeln GITHUB_TOKEN
error-token-step3 = 3. Starta om miniprogrammet
error-network = Nätverksfel uppstod
error-api = GitHub API fel
error-mark-read = Misslyckades med att markera som läst

# Anledningar för avisering
reason-assign = Du blev tilldelad
reason-author = Du skrev detta
reason-comment = Du kommenterade
reason-invitation = Du blev inbjuden
reason-manual = Du prenumererade
reason-mention = Du blev omnämnd
reason-review-requested = Granskning begärd
reason-security-alert = Säkerhetsvarning
reason-state-change = Status ändrad
reason-subscribed = Du prenumererar
reason-team-mention = Laget nämndes

# Tidsformatering
time-just-now = Just nu
time-minutes-ago = { $minutes ->
    [one] { $minutes } minut sedan
    *[other] { $minutes } minuter sedan
}
time-hours-ago = { $hours ->
    [one] { $hours } timme sedan
    *[other] { $hours } timmar sedan
}
time-days-ago = { $days ->
    [one] { $days } dag sedan
    *[other] { $days } dagar sedan
}

# Aviseringstyper
type-pull-request = Pull begäran
type-issue = Problem
type-release = Släpp
type-discussion = Diskussion
type-security-alert = Säkerhetsvarning
type-repository-invitation = Arkiv-inbjudan
