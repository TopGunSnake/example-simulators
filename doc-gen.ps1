cargo doc --workspace --no-deps
Remove-Item '.\docs\' -Recurse
Write-Output "<meta http-equiv=`"refresh`" content=`"0; url=example-simulators`">" > target/doc/index.html
Copy-Item target\doc .\docs -Recurse