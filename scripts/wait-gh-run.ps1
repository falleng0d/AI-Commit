param(
    [Parameter(Mandatory = $true)]
    [string]$RunId,

    [int]$PollSeconds = 10,

    [ValidateSet("completed", "in_progress", "queued")]
    [string]$DesiredStatus = "completed",

    [string]$DesiredConclusion = "success"
)

while ($true) {
    $json = gh run view $RunId --json databaseId,status,conclusion,url,workflowName,displayTitle,headBranch 2>$null

    if (-not $json) {
        Write-Error "Failed to load workflow run $RunId"
        exit 1
    }

    $run = $json | ConvertFrom-Json
    $timestamp = Get-Date -Format o

    if ($run.status -eq $DesiredStatus) {
        Write-Host "$timestamp [$($run.workflowName)] branch=$($run.headBranch) status=$($run.status) conclusion=$($run.conclusion)"

        if ($DesiredStatus -ne "completed") {
            exit 0
        }

        if ($run.conclusion -eq $DesiredConclusion) {
            exit 0
        }

        exit 1
    }

    if ($run.status -eq "completed" -and $run.conclusion -and $run.conclusion -ne $DesiredConclusion) {
        Write-Host "$timestamp [$($run.workflowName)] branch=$($run.headBranch) status=$($run.status) conclusion=$($run.conclusion)"
        Write-Error "Workflow run $RunId completed early with conclusion '$($run.conclusion)'"
        exit 1
    }

    Start-Sleep -Seconds $PollSeconds
}
