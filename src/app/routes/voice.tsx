import { useState, useRef } from 'react'
import { invoke } from '@tauri-apps/api/core'

// 定义 Rust 后端返回的数据结构
type VoiceResult = [Float32Array, number]

// 定义音频历史记录的数据结构
interface AudioRecord {
    id: number
    text: string
    audioData: Float32Array
    sampleRate: number
    timestamp: string // 用于显示生成时间
}

const DEFAULT_TEXT =
    'Hello Gemini, please generate this voice using the Rust backend.'

export function VoicePage() {
    // 状态
    const [text, setText] = useState(DEFAULT_TEXT)
    const [loading, setLoading] = useState(false)
    const [error, setError] = useState<string | null>(null)
    const [status, setStatus] = useState<string>('')

    // 🌟 新增：历史记录状态 (限制 5 条)
    const [history, setHistory] = useState<AudioRecord[]>([])

    // 引用：用于在播放时不重复创建 AudioContext
    const audioContextRef = useRef<AudioContext | null>(null)

    /**
     * 核心逻辑：播放 Float32Array 格式的原始音频数据
     * @param audioData 原始波形数据 (Float32Array)
     * @param sampleRate 采样率 (Hz)
     */
    const playAudio = (audioData: Float32Array, sampleRate: number) => {
        try {
            if (!audioContextRef.current) {
                audioContextRef.current = new (window.AudioContext ||
                    window.webkitAudioContext)()
            }
            const audioContext = audioContextRef.current

            // 1. 创建 AudioBuffer (单声道)
            const audioBuffer = audioContext.createBuffer(
                1,
                audioData.length,
                sampleRate
            )

            // 2. 拷贝数据
            audioBuffer.getChannelData(0).set(audioData)

            // 3. 创建 AudioSourceNode
            const source = audioContext.createBufferSource()
            source.buffer = audioBuffer
            source.connect(audioContext.destination)

            // 4. 播放
            source.start()
            setStatus('播放成功！')
        } catch (e) {
            console.error('播放音频失败:', e)
            setError(
                `音频播放失败: ${e instanceof Error ? e.message : String(e)}`
            )
            setStatus('播放失败')
        }
    }

    /**
     * 处理历史记录中的重新播放
     */
    const handlePlayHistory = (record: AudioRecord) => {
        if (loading) return // 如果正在生成新的语音，则禁止重复播放
        setStatus(`正在重新播放: "${record.text.substring(0, 30)}..."`)
        playAudio(record.audioData, record.sampleRate)
    }

    /**
     * 异步调用 Rust 后端 generate_voice 命令
     */
    async function generateAndPlayVoice() {
        setLoading(true) // 🌟 防抖：在调用开始时禁用按钮
        setError(null)
        setStatus('正在调用 Rust 后端生成...')

        const trimmedText = text.trim()
        if (!trimmedText) {
            setError('请输入文本！')
            setLoading(false)
            return
        }

        try {
            // 🚀 调用 Rust 命令
            const [wavArrayBuffer, sampleRate] = await invoke<VoiceResult>(
                'generate_voice',
                {
                    text: trimmedText
                }
            )

            // 1. 🌟 创建新的历史记录
            const newRecord: AudioRecord = {
                id: Date.now(),
                text: trimmedText,
                audioData: wavArrayBuffer,
                sampleRate: sampleRate,
                timestamp: new Date().toLocaleTimeString()
            }

            // 2. 🌟 更新历史记录状态 (新记录在前，并限制最多 5 条)
            setHistory((prevHistory) => [newRecord, ...prevHistory].slice(0, 5))

            // 3. 播放新生成的音频
            playAudio(wavArrayBuffer, sampleRate)
        } catch (err) {
            console.error('调用 Rust 接口失败:', err)
            setError(`生成失败: ${err}`)
            setStatus('生成失败')
        } finally {
            setLoading(false) // 🌟 防抖：在调用结束时重新启用按钮
        }
    }

    return (
        <div className="flex flex-col items-center justify-center min-h-screen p-6 bg-gray-50">
            <div className="w-full max-w-2xl bg-white p-8 rounded-xl shadow-2xl space-y-6">
                {/* 顶部导航和标题 */}
                <div className="flex justify-between items-center">
                    <a // 使用 <a> 替代 navigate 按钮
                        href="/"
                        className="text-indigo-600 hover:text-indigo-800 transition duration-150 font-medium"
                    >
                        ← 返回主页
                    </a>
                    <h1 className="text-3xl font-bold text-gray-800">
                        语音生成器 (TTS)
                    </h1>
                </div>

                {/* 状态和错误信息 */}
                <div className="space-y-2">
                    <p
                        className={`text-sm font-semibold ${error ? 'text-red-500' : 'text-gray-600'}`}
                    >
                        状态: {error || status || '等待输入...'}
                    </p>
                </div>

                {/* 输入框 */}
                <textarea
                    rows={4}
                    placeholder="在此输入英文文本..."
                    className="w-full p-4 border border-gray-300 rounded-lg focus:ring-indigo-500 focus:border-indigo-500 text-lg"
                    value={text}
                    onChange={(e) => setText(e.target.value)}
                    disabled={loading}
                />

                {/* 按钮：使用 loading 状态禁用 */}
                <button
                    onClick={generateAndPlayVoice}
                    disabled={loading} // 🌟 关键：防止多次点击
                    className={`w-full px-6 py-3 rounded-lg text-lg font-semibold transition duration-200 shadow-md ${
                        loading
                            ? 'bg-gray-400 cursor-not-allowed'
                            : 'bg-indigo-600 hover:bg-indigo-700 text-white'
                    }`}
                >
                    {loading ? '正在生成语音...' : '生成并播放音频'}
                </button>

                {/* 🌟 历史记录列表 */}
                {history.length > 0 && (
                    <div className="mt-8 pt-4 border-t border-gray-200 space-y-3">
                        <h2 className="text-xl font-semibold text-gray-700">
                            最近生成历史 (最多 5 条)
                        </h2>
                        <ul className="space-y-3">
                            {history.map((record) => (
                                <li
                                    key={record.id}
                                    className="flex justify-between items-center p-3 border border-gray-100 rounded-lg bg-gray-50 hover:bg-gray-100 transition-colors"
                                >
                                    <span className="text-sm text-gray-800 truncate mr-4">
                                        <span className="font-mono text-xs text-gray-500 mr-2">
                                            [{record.timestamp}]
                                        </span>
                                        {record.text}
                                    </span>
                                    <button
                                        onClick={() =>
                                            handlePlayHistory(record)
                                        }
                                        disabled={loading} // 生成新语音时禁止播放旧语音
                                        className="flex-shrink-0 px-3 py-1 text-xs font-medium rounded text-white bg-teal-500 hover:bg-teal-600 transition duration-150 disabled:bg-gray-400"
                                        title="点击重复播放"
                                    >
                                        重新播放
                                    </button>
                                </li>
                            ))}
                        </ul>
                    </div>
                )}

                <p className="text-xs text-gray-400 pt-2 text-center">
                    * 音频播放通过 Web Audio API 在内存中直接处理。
                </p>
            </div>
        </div>
    )
}

export const Component = VoicePage
