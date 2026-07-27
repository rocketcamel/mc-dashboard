pipeline {
    agent { label 'kaniko' }
    options { disableConcurrentBuilds() }

    stages {
        stage('checkout') {
            steps { checkout scm }
        }

        stage('build') {
          steps { container('kaniko') { sh '''/kaniko/executor \
            --context "$WORKSPACE" \
            --dockerfile "$WORKSPACE/Dockerfile" \
            --destination "registry.lucalise.ca/mc-dashboard:latest" \
            --snapshot-mode=redo \
            --use-new-run
          ''' } }
        }
    }
}
