pipeline {
    agent {
        kubernetes {
            yaml '''
apiVersion: v1
kind: Pod
spec:
  containers:
  - name: jnlp
    resources:
      requests:
        cpu: 50m
        memory: 256Mi
      limits:
        cpu: 500m
        memory: 512Mi
  - name: kaniko
    image: gcr.io/kaniko-project/executor:v1.23.2-debug
    command:
    - /busybox/sleep
    args:
    - "86400"
    resources:
      requests:
        cpu: 100m
        memory: 512Mi
      limits:
        cpu: 1000m
        memory: 2Gi
    volumeMounts:
    - name: docker-config
      mountPath: /kaniko/.docker
  volumes:
  - name: docker-config
    secret:
      secretName: nexus-registry-credentials
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
                        def gitCommit = env.GIT_COMMIT?.take(7) ?: 'unknown'
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
