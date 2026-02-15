@Library('homelab-jenkins-library@main') _

pipeline {
    agent {
        kubernetes {
            yaml homelab.podTemplate('kaniko')
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
                        def gitCommit = homelab.gitShortCommit()
                        sh """
                            /kaniko/executor \
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
        failure {
            script {
                homelab.notifyDiscord(status: 'FAILURE')
            }
        }
        success {
            script {
                if (env.BRANCH_NAME == 'main' || env.BRANCH_NAME == 'master' || env.TAG_NAME) {
                    echo "Image pushed to: ${REGISTRY}/${IMAGE_NAME}"
                }
            }
        }
    }
}
