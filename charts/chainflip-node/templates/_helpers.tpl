{{/*
Define name for chainflip-node
*/}}
{{- define "chainflip-node.fullname" -}}
{{ .Release.Name }}-node
{{- end }}

{{/*
Define name for chainflip-engine
*/}}
{{- define "chainflip-engine.fullname" -}}
{{ .Release.Name }}-engine
{{- end }}

{{/*
Create chart name and version as used by the chart label.
*/}}
{{- define "chainflip-node.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "chainflip-node.labels" -}}
{{ include "chainflip-node.selectorLabels" . }}
chainflip.io/unit: chainflip-node
{{- end }}

{{/*
Common labels for engine
*/}}
{{- define "chainflip-engine.labels" -}}
{{ include "chainflip-engine.selectorLabels" . }}
chainflip.io/unit: chainflip-engine
{{- end }}

{{/*
Selector labels
*/}}
{{- define "chainflip-node.selectorLabels" -}}
app.kubernetes.io/name: {{ include "chainflip-node.fullname" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Selector labels for the engine
*/}}
{{- define "chainflip-engine.selectorLabels" -}}
app.kubernetes.io/name: {{ include "chainflip-engine.fullname" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Create the name of the service account to use
*/}}
{{- define "chainflip-node.serviceAccountName" -}}
{{- if .Values.serviceAccount.create }}
{{- default (include "chainflip-node.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}
