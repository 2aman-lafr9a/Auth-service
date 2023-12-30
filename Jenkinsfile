pipeline {
  agent {
    dockerfile {
      filename 'auth_service'
    }

  }
  stages {
    stage('error') {
      steps {
        sh 'docker compose up -d'
      }
    }

  }
}