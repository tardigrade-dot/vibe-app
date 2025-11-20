import { useState } from 'react' // 导入 useState 用于状态管理
import { invoke } from '@tauri-apps/api/core' // 导入 invoke 用于调用 Rust 后端
import env from '@/config/env'
import BuiltWith from '@/features/built-with'
import GithubStarButton from '@/features/github-star-button'

// Tauri 接口调用的默认值
const DEFAULT_A = 10
const DEFAULT_B = 5

export function HomePage() {
    // 状态管理：用于输入和显示结果
    const [numA, setNumA] = useState(DEFAULT_A)
    const [numB, setNumB] = useState(DEFAULT_B)
    const [result, setResult] = useState<number | null>(null)
    const [loading, setLoading] = useState(false)
    const [error, setError] = useState<string | null>(null)
    /**
     * 异步调用 Rust 后端 add_method 命令
     */
    async function callAddMethod() {
        setLoading(true)
        setResult(null)
        setError(null)

        try {
            // 确保输入是数字
            const a = parseInt(String(numA) || '0')
            const b = parseInt(String(numB) || '0')

            // 🚀 调用 Rust 命令
            const sum = await invoke<number>('add_method', {
                a: a,
                b: b
            })

            setResult(sum)
            console.log(`Rust 后端返回的结果: ${sum}`)
        } catch (err) {
            // 捕获 Rust 接口返回的错误
            console.error('调用 Rust 接口失败:', err)
            setError(`计算失败: ${err}`)
        } finally {
            setLoading(false)
        }
    }

    return (
        <div className="flex h-screen">
            <div className="m-auto text-center space-y-6 p-4">
                <div className="space-y-3">
                    <BuiltWith />
                    <h1 className="text-3xl items-center font-bold">
                        Welcome to Tauri React template!
                    </h1>
                    <a
                        href="/voice" // 🌟 设置目标路由
                        className="inline-block px-4 py-2 rounded text-white bg-green-600 hover:bg-green-700 transition duration-150 font-medium shadow-md"
                    >
                        前往语音生成界面
                    </a>
                    <p className="text-gray-600">
                        这是一个 Tauri React 模版，现在包含了 Rust
                        接口调用示例。
                    </p>
                    <p className="text-sm text-gray-500">
                        (Env variable: {env.API_URL})
                    </p>
                </div>

                {/* --- 🌟 Rust 接口调用部分 --- */}
                <div className="mt-8 p-6 border border-gray-200 rounded-lg shadow-md space-y-4 bg-white">
                    <h2 className="text-xl font-semibold text-indigo-600">
                        Rust 命令调用 (`add_method`)
                    </h2>

                    <div className="flex justify-center space-x-2 items-center">
                        <input
                            type="number"
                            placeholder="数字 A"
                            className="p-2 border rounded w-24 text-center"
                            value={numA}
                            onChange={(e) => setNumA(parseInt(e.target.value))}
                        />
                        <span className="text-2xl font-bold">+</span>
                        <input
                            type="number"
                            placeholder="数字 B"
                            className="p-2 border rounded w-24 text-center"
                            value={numB}
                            onChange={(e) => setNumB(parseInt(e.target.value))}
                        />

                        <button
                            onClick={callAddMethod}
                            disabled={loading}
                            className={`px-4 py-2 rounded text-white transition duration-150 ${
                                loading
                                    ? 'bg-indigo-400 cursor-not-allowed'
                                    : 'bg-indigo-600 hover:bg-indigo-700'
                            }`}
                        >
                            {loading ? '计算中...' : '调用 Rust'}
                        </button>
                    </div>

                    <div className="mt-4">
                        {result !== null && (
                            <p className="text-lg font-bold text-green-600">
                                结果: {numA} + {numB} = {result}
                            </p>
                        )}
                        {error && (
                            <p className="text-red-500 text-sm">{error}</p>
                        )}
                        {!loading && result === null && !error && (
                            <p className="text-gray-500 text-sm">
                                点击按钮进行计算
                            </p>
                        )}
                    </div>
                </div>
                {/* ------------------------------- */}

                <div className="pt-4">
                    <GithubStarButton />
                </div>
            </div>
        </div>
    )
}

// Necessary for react router to lazy load.
export const Component = HomePage
