@{
    ModuleVersion     = '0.1.0'
    Author            = 'Nicolas Arnaud-Cormos'
    Description       = 'A simple PowerShell for the use tool'
    Copyright         = 'Nicolas Arnaud-Cormos'
    RootModule        = 'posh-use.psm1'

    PrivateData       = @{
        PSData = @{
            LicenseUri = 'https://opensource.org/licenses/MIT'
            ProjectUri = 'https://github.com/narnaud/use'
        }
    }

    FunctionsToExport = @(
        'Use-Environment'
    )
    AliasesToExport   = @(
        'use'
    )
}
