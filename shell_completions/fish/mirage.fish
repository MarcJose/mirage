complete -c mirage -l connection-timeout -r
complete -c mirage -l download-timeout -r
complete -c mirage -l cache-timeout -r
complete -c mirage -l url -r
complete -c mirage -l save -r
complete -c mirage -l sort -r -f -a "age\t''
rate\t''
country\t''
score\t''
delay\t''
duration\t''
duration-std\t''"
complete -c mirage -l threads -r
complete -c mirage -s a -l age -r
complete -c mirage -l delay -r
complete -c mirage -s c -l country -r
complete -c mirage -s f -l fastest -r
complete -c mirage -s i -l include -r
complete -c mirage -s x -l exclude -r
complete -c mirage -s l -l latest -r
complete -c mirage -l score -r
complete -c mirage -s n -l number -r
complete -c mirage -s p -l protocol -r
complete -c mirage -l completion-percent -r
complete -c mirage -l list-countries
complete -c mirage -l verbose
complete -c mirage -s q -l quiet
complete -c mirage -l info
complete -c mirage -l isos
complete -c mirage -l ipv4
complete -c mirage -l ipv6
complete -c mirage -l clear-cache -d 'Clear the persistent cache'
complete -c mirage -l cache-info -d 'Show cache information'
complete -c mirage -s h -l help -d 'Print help'
complete -c mirage -s V -l version -d 'Print version'
