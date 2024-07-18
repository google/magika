rule Rule_485729_77379 {
  strings:
    $s1 = "HKEY_LOCAL_MACHINE\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run"
    $s2 = "Win32_Process"
    $s3 = "Create" wide
  condition:
    $s1 and ($s2 and $s3)
  meta:
    author = "CyberThreatResearch"
    date = "2019-09-23"
    tags = "malware, persistence, registry"
}
