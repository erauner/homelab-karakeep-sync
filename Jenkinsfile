pipeline {
    agent {
        kubernetes {
            yaml '''
apiVersion: v1
kind: Pod
spec:
  containers:
  - name: kaniko
    image: gcr.io/kaniko-project/executor:v1.23.2-debug
    command:
    - /busybox/sleep
    args:
    - "86400"
    volumeMounts:
    - name: docker-config
      mountPath: /kaniko/.docker
  volumes:
  - name: docker-config
    secret:
      secretName: nexus-docker-config
      items:
      - key: .dockerconfigjson
        path: config.json
'''
        }
    }

    environment {
        REGISTRY = 'docker.nexus.erauner.dev'
        IMAGE_NAME = 'homelab/karakeep-sync'
    }

    stages {
        stage('Build & Push Image') {
            when {
                anyOf {
                    branch 'main'
                    branch 'master'
                    tag pattern: 'v*', comparator: 'GLOB'
                }
            }
            steps {
                container('kaniko') {
                    script {
                        def imageTag = env.TAG_NAME ?: 'latest'
                        def gitCommit = sh(script: 'git rev-parse --short HEAD', returnStdout: true).trim()
                        sh """
                            /kaniko/executor \
                                --context=\${WORKSPACE} \
                                --dockerfile=\${WORKSPACE}/Dockerfile \
                                --destination=${REGISTRY}/${IMAGE_NAME}:${imageTag} \
                                --destination=${REGISTRY}/${IMAGE_NAME}:${gitCommit} \
                                --cache=true \
                                --cache-repo=${REGISTRY}/${IMAGE_NAME}/cache
                        """
                    }
                }
            }
        }
    }

    post {
        success {
            script {
                if (env.BRANCH_NAME == 'main' || env.BRANCH_NAME == 'master' || env.TAG_NAME) {
                    echo "Image pushed to: ${REGISTRY}/${IMAGE_NAME}"
                }
            }
        }
    }
}
